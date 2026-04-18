use serde::Serialize;
use std::fmt;

use crate::{config::ActionType, validation_error::ValidationError};

#[derive(Serialize, Default)]
pub struct ValidationState {
    #[serde(rename = "actionType")]
    pub action_type: Option<ActionType>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub errors: Vec<ValidationError>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationError>,
}

impl fmt::Debug for ValidationState {
    // Hand-rolled so an empty `warnings` vec doesn't show up in stderr
    // snapshots taken before the field existed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("ValidationState");
        d.field("action_type", &self.action_type);
        d.field("file_path", &self.file_path);
        d.field("errors", &self.errors);
        if !self.warnings.is_empty() {
            d.field("warnings", &self.warnings);
        }
        d.finish()
    }
}

impl ValidationState {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl From<valico::json_schema::ValidationState> for ValidationState {
    fn from(state: valico::json_schema::ValidationState) -> Self {
        (&state).into()
    }
}

impl From<&valico::json_schema::ValidationState> for ValidationState {
    fn from(state: &valico::json_schema::ValidationState) -> Self {
        ValidationState {
            errors: state.errors.iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
