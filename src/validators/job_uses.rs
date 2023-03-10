use regex::{Regex, Captures};

use crate::validation_state::ValidationState;
use crate::validators::models;

/// A simple enum providing exhaustive matching to [`_action_type`].
enum ActionType {
    Action,
    Docker,
    Path,
}

/// Validates all `jobs.<job_id>.steps[*].uses` values in the provided workflow file(s). This
/// validator has remote checks which will only run if the `remote-checks` feature flag is enabled.
/// If the feature flag is disabled, then this validate confirms the shape of the uses statement
/// matches GitHub's expected format
/// ([more here](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idstepsuses)).
///
/// If the feature flag is enabled, the above checks will be validated in addition to the remote
/// checks. The [`models::Action`] and [`models::Docker`] structs both implement remote checks.
///
/// # Arguments
/// * doc - The parsed workflow document, to be validated.
/// * state - The [`ValidationState`] to which will be used to provide validation errors.
pub fn validate(doc: &serde_json::Value, state: &mut ValidationState) {
    // If this regex doesn't compile, that should be considered a compile-time
    // error. As such, we should unwrap to purposefully panic in the event of
    // a malformed regex.
    let pattern = vec![
        models::Action::PATTERN,
        models::Docker::PATTERN,
        models::Path::PATTERN,
    ].join("|");
    let r = Regex::new(format!(r"(?x)^{pattern}$").as_str()).unwrap();

    let default_map = &serde_json::Map::<String, serde_json::Value>::new();
    let jobs_step_uses = doc["jobs"]
        .as_object()
        .unwrap_or(default_map)
        .iter()
        .flat_map(|(job_name, job)| {
            Some((job_name, job["steps"].as_array()?.iter()))
        })
        .flat_map(|(job_name, steps)| {
            steps.map(|step| {
                    Some((job_name.to_owned(), step["uses"].as_str()?))
            })
        })
        .filter_map(|o| o)
        .collect::<Vec<_>>();

    for (job_name, uses) in jobs_step_uses {
        let origin = format!("jobs/{job_name}/steps/uses/{uses}");
        let captures_op = &r.captures(uses);

        let uses_type = vec![
            ActionType::Action, ActionType::Docker, ActionType::Path,
        ]
            .into_iter()
            .find_map(|action_type| {
                Some(_action_type(action_type, &origin, captures_op.as_ref()?))
            })
            .unwrap_or(Box::new(models::Invalid{
                uses: String::from(uses),
                origin: origin.to_owned(),
            }));

        if let Err(v) = uses_type.validate() {
            state.errors.push(v);
        }
    }
}

/// Matches on the provided `action_type` and extracts the `captures` named capture groups for that
/// implementation of the `Uses` trait.
///
/// # Arguments
/// * `action_type` - An enum indicating if the action type is `Action`, `Docker`, or `Path`.
/// * `origin` - The origin path of the `uses` string being validated from the workflow.
/// * `captures` - The capture group which matched the validation regex.
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
