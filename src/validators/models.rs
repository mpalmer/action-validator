use crate::validation_error::ValidationError;

#[cfg(feature="remote-checks")]
use reqwest::{StatusCode, blocking::Client};

pub trait Uses<'a>: std::fmt::Debug {
    fn validate(&self) -> Result<(), ValidationError>;
}

pub trait Other {
}

#[derive(Debug)]
pub struct Invalid {
    pub uses: String,
    pub origin: String,
}

impl Uses<'_> for Invalid {
    fn validate(&self) -> Result<(), ValidationError> { 
        Ok(())
    }
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

#[cfg(feature="remote-checks")]
fn _get_request(url: String) -> Result<reqwest::blocking::Response, reqwest::Error> {
    Client::new().get(url).send()
}

impl Uses<'_> for Action {
    #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

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

impl Uses<'_> for Docker {
    #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

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
                // In this case, pull access requires authorized. For now, we should assume
                // this the tag is valid. We could perform authentication, but that would
                // require user creds and add a whole lot of scope to this feature. For now, an
                // unauthenticated requests means the image exists, and that is sufficient.
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

        // How do we handle failed requests? If the user has the `remote-checks` feature enable,
        // they likely want to know about these failures. We can mark this as a validation error.
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

impl Uses<'_> for Path {
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

