use regex::Regex;

use crate::validation_error::ValidationError;
use crate::validation_state::ValidationState;

use crate::validators::models::{Action, Uses, Invalid};


pub fn validate(doc: &serde_json::Value, state: &mut ValidationState) -> Option<()> {
    // If this regex doesn't compile, that should be considered a compile-time
    // error. As such, we should unwrap to purposefully panic in the event of
    // a malformed regex.
    let r = Regex::new(r"(?x)^
    (?P<Action>.{0}(?P<owner>[^/]*)/(?P<repo>[^/]*)(/(?P<path>.*))?@(?P<ref>.*))|
    (?P<Path>.{0}\./([^/]+))|
    (?P<Docker>.{0}docker://(?P<image>.*)(:(?P<tag>.*))?)|
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
        let uses_op: Option<Box<dyn Uses>> = vec![
            "Action", "Path", "Docker",
        ]
        .into_iter()
        .find_map::<Box<dyn Uses>, _>(|name| {
            // If the regex didn't match any of them,
            // then it's an error.
            matched.name(name)?;

            let origin = format!("jobs/{job_name}/steps/uses");
            let uses = String::from(&matched[0]);
            match name {
                "Path" => {
                    Some(Box::new(Invalid{uses, origin}))
                },
                "Docker" => {
                    Some(Box::new(Invalid{uses, origin}))
                },
                "Action" => Some(Box::new(Action {
                    owner: String::from(matched.name("owner")?.as_str()),
                    repo: String::from(matched.name("repo")?.as_str()),
                    path: matched.name("path").map(|m| String::from(m.as_str())),
                    reference: String::from(matched.name("ref")?.as_str()),
                    origin,
                })),
                _ => {
                    Some(Box::new(Invalid{uses, origin}))
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

fn validate_remote_checks(uses: Box<dyn Uses>, state: &mut ValidationState) -> () {
    if !cfg!(feature="remote-checks") {
        return ();
    }
    if let Err(v) = uses.validate() {
        state.errors.push(v);
    }
}
