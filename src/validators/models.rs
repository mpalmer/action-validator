use crate::validation_error::ValidationError;

#[cfg(feature="remote-checks")]
use reqwest::{blocking::Response, StatusCode, blocking::Client};

/// A method to perform a synchronous get request and return it's result.
///
/// # Arguments
/// * url - The FQDN where the GET request will be sent.
#[cfg(feature="remote-checks")]
fn _get_request(url: String) -> Result<Response, reqwest::Error> {
    Client::new().get(url).send()
}

/// This trait provides a common method for all action type implementations to validate said action
/// type. An implementation of `validate` should return a [`ValidationError`] if one were to occur,
/// the validation error should be pushed onto the validation error stack.
pub trait Uses<'a>: std::fmt::Debug {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug)]
pub struct Action {
    pub uses: String,
    pub origin: String,
    pub owner: String,
    pub repo: String,
    pub path: Option<String>,
    pub reference: String,
}

impl Action {
    pub const PATTERN: &str = r"(?P<Action>.{0}(?P<owner>[^/]*)/(?P<repo>[^/]*)(/(?P<path>.*))?@(?P<ref>.*))";
}

impl Uses<'_> for Action {
    #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    /// Remote-check validation logic for GitHub Actions. If the destructured action is able to be
    /// retrieved from the repositories tree, then the action exists. Otherwise, the action is
    /// formatted properly but does not exist.
    #[cfg(feature="remote-checks")]
    fn validate(&self) -> Result<(), ValidationError> {
        let response = _get_request(format!(
                    "https://github.com/{0}/{1}/tree/{2}/{3}",
                    self.owner,
                    self.repo,
                    self.reference,
                    self.path.as_ref().unwrap_or(&String::new()),
            ));
        if let Some(r) = response.ok() {
            if r.status() == 200 {
                return Ok(());
            }
            return Err(ValidationError::Unknown { 
                code: "action_not_found".into(),
                detail: Some(format!("Could not find action: {}", self.uses)),
                path: self.origin.to_owned(),
                title: "Action Not Found".into(),
            });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Docker {
    pub uses: String,
    pub origin: String,
    pub image: String,
    pub url: Option<String>,
    pub tag: Option<String>,
}

impl Docker {
    pub const PATTERN: &str = r"(?P<Docker>.{0}(?:docker://)(?P<url>([^/:]+)\.([^/:]+)/)?(?P<image>[^:]+)(?::(?P<tag>.+))?)";
}

impl Uses<'_> for Docker {
    #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    /// Remote-check validation logic for Docker images. If the destructured image is able to be
    /// retrieved from Docker Hub or using the registries v2 endpoints, then the image exists. If
    /// the image results in a 401 (UNAUTHORIZED), the image _could_ exist. In this case,
    /// action-validator will assume the image exists.
    #[cfg(feature="remote-checks")]
    fn validate(&self) -> Result<(), ValidationError> {
        let mut image = self.image.to_owned();

        // If there's no prefix and the image is in docker hub, then it means the image is a
        // library image.
        if self.url.is_none() && !image.contains("/") { 
            image = format!("library/{image}");
        }

        let url = match (self.url.as_ref(), self.tag.as_ref()) {
            // Lookup V2 protocol tag
            (Some(url), Some(tag)) => format!("https://{url}/v2/{image}/manifests/{tag}"),
            // Lookup V2 protocol image
            (Some(url), None) => format!("https://{url}/v2/{image}/tags/list"),
            // Lookup docker hub tag
            (None, Some(tag)) => {
                format!("https://registry.hub.docker.com/v2/repositories/{image}/tags/{tag}")
            },
            // Lookup docker hub image
            (None, None) => {
                format!("https://registry.hub.docker.com/v2/repositories/{image}")
            },
        };

        if let Some(r) = _get_request(url).ok() {
            let status = r.status();
            if status == StatusCode::OK {
                return Ok(());
            } else if status == StatusCode::UNAUTHORIZED {
                // In this case, pull access requires authorization. There are simple cases that
                // only require the bearer token retrieval followed by manifest access, but there
                // are also others that require user credentials. For now, we should assume that
                // the tag tag is valid. We could perform authentication, but that would requrie
                // user creds and adds a whole lot of scope to this feature.
                return Ok(());
            } else if status == StatusCode::NOT_FOUND {
                return Err(ValidationError::Unknown { 
                    code: "docker_action_not_found".into(),
                    detail: Some(format!("Could not find the docker action: {}", self.uses)),
                    path: self.origin.to_owned(),
                    title: "Docker Action Not Found".into(),
                });
            } else {
                return Err(ValidationError::Unknown { 
                    code: "unexpected_server_response".into(),
                    detail: Some(format!("Unexpected server response: {}", status)),
                    path: self.origin.to_owned(),
                    title: "Docker Action Not Found".into(),
                });
            }
        }

        // The remote-check request failed. If the user has the `remote-checks` feature enable,
        // they likely want to know about these failures. We can mark this as an error.
        Err(ValidationError::Unknown { 
            code: "unexpected_server_response".into(),
            detail: Some(format!("Could not find docker action: {}", self.uses)),
            path: self.origin.to_owned(),
            title: "Docker Action Not Found".into(),
        })
    }
}

#[derive(Debug)]
pub struct Path {
    pub uses: String,
    pub origin: String,
}

impl Path {
    pub const PATTERN: &str = r"(?P<Path>.{0}\./([^/]+/?)+)";
}

impl Uses<'_> for Path {
    /// Validates that the supplied path exists.
    fn validate(&self) -> Result<(), ValidationError> {
        if std::path::Path::new(self.uses.as_str()).exists() {
            return Ok(());
        }
        Err(ValidationError::Unknown { 
            code: "action_not_found".into(),
            detail: Some(format!("The action path does not exist: {}", self.uses)),
            path: self.origin.to_owned(),
            title: "Action Not Found".into(),
        })
    }
}

#[derive(Debug)]
pub struct Invalid {
    pub uses: String,
    pub origin: String,
}

impl Uses<'_> for Invalid {
    /// If the provided action is not a [`Docker`], [`Action`], or [`Path`], the action is invalid.
    /// When this [`Uses`] implementation is created, the validation will always return a
    /// validation error.
    fn validate(&self) -> Result<(), ValidationError> { 
        let uses = self.uses.to_owned();
        Err(ValidationError::InvalidGlob {
            code: "invalid_uses".into(),
            detail: Some(format!("The `uses` {uses} is invalid.")),
            path:  self.origin.to_owned(),
            title: "Invalid Uses".into(),
        })
    }
}
