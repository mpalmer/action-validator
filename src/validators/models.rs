use core::fmt;

use crate::validation_error::ValidationError;

#[cfg(feature="remote-checks")]
use reqwest::blocking::Client;

pub trait Uses<'a>: std::fmt::Debug {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug)]
pub struct Invalid {
    pub uses: String,
    pub origin: String,
}

impl Uses<'_> for Invalid {
    fn validate(&self) -> Result<(), ValidationError> { 
        println!("{:#?}", self);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Action {
    pub owner: String,
    pub repo: String,
    pub path: Option<String>,
    pub reference: String,
    pub origin: String,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut p = vec![self.owner.as_ref(), self.repo.as_ref()];
        if let Some(path) = self.path.as_ref() {
            p.push(path.as_str());
        }
        write!(f, "{}@{}", p.join("/"), self.reference.as_str())
    }
}


impl Uses<'_> for Action {
    #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    #[cfg(feature="remote-checks")]
    fn validate(&self) -> Result<(), ValidationError> {
        let request = Client::new()
            .get(format!(
                    "https://github.com/{0}/{1}/tree/{2}/{3}",
                    self.owner,
                    self.repo,
                    self.reference,
                    self.path.as_ref().unwrap_or(&String::new()),
            ));
        let response = request.send();
        if let Some(r) = response.ok() {
            if r.status() == 200 {
                return Ok(());
            }
            return Err(ValidationError::Unknown { 
                code: "action_not_found".into(),
                detail: Some(format!("Could not find action: {}", self)),
                path: self.origin.to_owned(),
                title: "Action Not Found".into(),
            });
        }

        Ok(())
    }
}
