use crate::validation_error::ValidationError;

#[cfg(feature="remote-checks")]
use reqwest::blocking::Client;

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
        println!("{:#?}", self);
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
    // #[cfg(not(feature="remote-checks"))]
    fn validate(&self) -> Result<(), ValidationError> {
        println!("{:#?}", self);
        Ok(())
    }

    // #[cfg(feature="remote-checks")]
    // fn validate(&self) -> Result<(), ValidationError> {
    //     let request = Client::new()
    //         .get(format!(
    //                 "https://github.com/{0}/{1}/tree/{2}/{3}",
    //                 self.owner,
    //                 self.repo,
    //                 self.reference,
    //                 self.path.as_ref().unwrap_or(&String::new()),
    //         ));
    //     let response = request.send();
    //     if let Some(r) = response.ok() {
    //         if r.status() == 200 {
    //             return Ok(());
    //         }
    //         return Err(ValidationError::Unknown { 
    //             code: "action_not_found".into(),
    //             detail: Some(format!("Could not find action: {}", self.uses)),
    //             path: self.origin.to_owned(),
    //             title: "Action Not Found".into(),
    //         });
    //     }

    //     Ok(())
    // }

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

