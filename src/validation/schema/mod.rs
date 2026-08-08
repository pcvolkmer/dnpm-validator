use crate::SchemaType;
use crate::validation::{ValidationError, map_to_validation_error};
use tree_sitter::Parser;

pub fn validate(
    json: &str,
    schema: SchemaType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<serde_json::Value>(json)?;

    let schema_str = match schema {
        SchemaType::MTB => include_str!("../schema/mtb.json"),
        SchemaType::RD => include_str!("../schema/rd.json"),
    };

    let validator =
        jsonschema::validator_for(&serde_json::from_str::<serde_json::Value>(schema_str)?)?;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let errors = validator
        .iter_errors(&value.clone())
        .map(|error| {
            let err = error.to_string();
            let err_path = error.instance_path().to_string();
            map_to_validation_error((err_path, err), json, &mut parser)
        })
        .collect::<Vec<_>>();

    Ok(errors)
}
