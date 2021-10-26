use serde_json::Value;

pub fn validate_as_action(doc: &Value) -> bool {
    validate_with_schema(doc, include_bytes!("github-action.json"))
}

pub fn validate_as_workflow(doc: &Value) -> bool {
    validate_with_schema(doc, include_bytes!("github-workflow.json"))
}

fn validate_with_schema(doc: &Value, schema: &[u8]) -> bool {
    let schema_json: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(schema).as_ref()).unwrap();
    let mut scope = valico::json_schema::Scope::new();
    let validator = scope.compile_and_return(schema_json, false).unwrap();

    let state = validator.validate(doc);

    if state.is_valid() {
        true
    } else {
        eprintln!("Validation failed: {:#?}", state);
        false
    }
}
