use regex::Regex;

use crate::validation_error::ValidationError;
use crate::validation_state::ValidationState;

use crate::validators::models;


pub fn validate(doc: &serde_json::Value, state: &mut ValidationState) -> Option<()> {
    // If this regex doesn't compile, that should be considered a compile-time
    // error. As such, we should unwrap to purposefully panic in the event of
    // a malformed regex.
    let r = Regex::new(r"(?x)^
    (?P<Action>.{0}(?P<owner>[^/]*)/(?P<repo>[^/]*)(/(?P<path>.*))?@(?P<ref>.*))|
    (?P<Path>.{0}\./([^/]+/?)+)|
    (?P<Docker>.{0}(?:docker://)(?P<url>([^:]+)/)?(?P<image>[^/:]+)(?::(?P<tag>.+))?)|
    $").unwrap();

    let all_uses = doc["jobs"]
        .as_object()?
        .iter()
        .map(|(job_name, job)| {
            job["steps"].as_array().map(|steps| {
                steps
                .iter()
                .map(|step| {
                    Some((job_name, step["uses"].as_str()?))
                })
                .flatten()
                .collect::<Vec<_>>()
            })
            .unwrap_or(vec![])
    })
    .flatten()
    .collect::<Vec<_>>();
    for (job_name, uses) in all_uses {
        let matched = r.captures(uses)?;
        let uses_op: Option<Box<dyn models::Uses>> = vec![
            "Action", "Path", "Docker",
        ]
        .into_iter()
        .find_map::<Box<dyn models::Uses>, _>(|name| {
            // If the regex didn't match any of them,
            // then it's an error.
            matched.name(name)?;

            let origin = format!("jobs/{job_name}/steps/uses");
            let uses = String::from(&matched[0]);
            match name {
                "Path" => {
                    Some(Box::new(models::Path{uses, origin}))
                },
                "Docker" => {
                    let image = String::from(matched.name("image").unwrap().as_str());
                    let url = matched.name("url").map(|v| String::from(v.as_str()));
                    let tag = matched.name("tag").map(|v| String::from(v.as_str()));
                    Some(Box::new(models::Docker {
                        uses,
                        origin,
                        image,
                        url,
                        tag,
                    }))
                },
                "Action" => {
                    let owner = String::from(matched.name("owner").unwrap().as_str());
                    let repo = String::from(matched.name("repo").unwrap().as_str());
                    let path = matched.name("path").map(|v| String::from(v.as_str()));
                    let reference = String::from(matched.name("ref").unwrap().as_str());
                    Some(Box::new(models::Action {
                        uses,
                        origin,
                        owner,
                        repo,
                        path,
                        reference,
                    }))
                },
                _ => {
                    Some(Box::new(models::Invalid{uses, origin}))
                 },
            }
        });

        if let Some(uses) = uses_op {
            if cfg!(feature="remote-checks") {
                validate_remote_checks(uses, state);
            }
        } else {
            state.errors.push(ValidationError::InvalidGlob {
                code: "invalid_uses".into(),
                detail: Some(format!("The `uses` {uses} is invalid.")),
                path:  "".into(),
                title: "".into(),
            });
        }
    }
    let _ = state;
    let _ = validate_remote_checks;


    Some(())
}

fn validate_remote_checks(uses: Box<dyn models::Uses>, state: &mut ValidationState) -> () {
    if !cfg!(feature="remote-checks") {
        return ();
    }
    if let Err(v) = uses.validate() {
        state.errors.push(v);
    }
}
