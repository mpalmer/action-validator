use regex::{Regex, Captures};

use crate::validation_state::ValidationState;

use crate::validators::models;

use super::models::Invalid;


pub fn validate(doc: &serde_json::Value, state: &mut ValidationState) -> Option<()> {
    // If this regex doesn't compile, that should be considered a compile-time
    // error. As such, we should unwrap to purposefully panic in the event of
    // a malformed regex.
    let pattern = vec![
        models::Action::PATTERN,
        models::Docker::PATTERN,
        models::Path::PATTERN,
    ].join("|");
    let r = Regex::new(format!(r"(?x)^{pattern}$").as_str()).unwrap();

    let jobs_step_uses = doc["jobs"]
        .as_object()?
        .iter()
        .map(|(job_name, job)| {
            job["steps"].as_array().map(|steps| {
                steps
                .iter()
                .map(|step| Some((job_name, step["uses"].as_str()?)))
                .flatten()
                .collect::<Vec<_>>()
            })
            .unwrap_or(vec![])
        })
        .flatten()
        .collect::<Vec<_>>();

    for (job_name, uses) in jobs_step_uses {
        let origin = format!("jobs/{job_name}/steps/uses/{uses}");
        let captures_op = &r.captures(uses);

        let uses_type = vec![ActionType::Action, ActionType::Docker, ActionType::Path]
            .into_iter()
            .find_map(|action_type| {
                if let Some(captures) = captures_op {
                    Some(_action_type(action_type, &origin, &captures))
                } else {
                    Some(Box::new(Invalid{
                        uses: String::from(uses),
                        origin: origin.to_owned(),
                    }))
                }
            })
            .unwrap_or(Box::new(Invalid{
                uses: String::from(uses),
                origin: origin.to_owned(),
            }));

        if let Err(v) = uses_type.validate() {
            state.errors.push(v);
        }
    }

    Some(())
}

enum ActionType {
    Action,
    Docker,
    Path,
}

fn _action_type<'a>(
    action_type: ActionType,
    origin: &String,
    captures: &Captures<'a>,
) -> Box<dyn models::Uses<'a>> {
    let origin = origin.to_owned();
    let uses = String::from(&captures[0]);
    match action_type {
        ActionType::Path => Box::new(models::Path{uses, origin}),
        ActionType::Docker => Box::new(models::Docker {
            uses,
            origin,
            // The `image` capture group is guranteed to exist when `Docker` does.
            image: String::from(captures.name("image").unwrap().as_str()),
            url: captures.name("url").map(|v| String::from(v.as_str())),
            tag: captures.name("tag").map(|v| String::from(v.as_str())),
        }),
        ActionType::Action => Box::new(models::Action {
            uses,
            origin,
            // The `owner`, `repo`, and `reference` capture groups are guranteed
            // to exist when `Action` does.
            owner: String::from(captures.name("owner").unwrap().as_str()),
            repo: String::from(captures.name("repo").unwrap().as_str()),
            reference: String::from(captures.name("ref").unwrap().as_str()),
            path: captures.name("path").map(|v| String::from(v.as_str())),
        }),
    }
}
