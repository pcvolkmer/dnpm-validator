mod reference;
pub mod schema;

use crate::SchemaType;
use std::cmp::Ordering;
use tree_sitter::{Node, Parser};

pub enum ValidationError {
    PosError {
        line: usize,
        column: usize,
        message: String,
        path: String,
    },
    Error {
        message: String,
        path: String,
    },
}

pub fn validate(
    json: &str,
    schema: SchemaType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    if serde_json::from_str::<serde_json::Value>(json).is_err() {
        return Ok(vec![ValidationError::Error {
            message: "No JSON file".to_string(),
            path: String::new(),
        }]);
    }

    let mut errors = schema::validate(json, schema)?;
    errors.append(&mut reference::validate(json, schema)?);

    errors.sort_by(|e1, e2| match e1 {
        ValidationError::Error { .. } => Ordering::Less,
        ValidationError::PosError { column, .. } => match e2 {
            ValidationError::Error { .. } => Ordering::Greater,
            ValidationError::PosError {
                column: column2, ..
            } => column.cmp(column2),
        },
    });

    errors.sort_by(|e1, e2| match e1 {
        ValidationError::Error { .. } => Ordering::Less,
        ValidationError::PosError { line, .. } => match e2 {
            ValidationError::Error { .. } => Ordering::Greater,
            ValidationError::PosError { line: line2, .. } => line.cmp(line2),
        },
    });

    Ok(errors)
}

fn find_node<'a>(node: Node<'a>, json: &'a str, path: &[&str]) -> Option<Node<'a>> {
    let mut cursor = node.walk();

    if path.is_empty() {
        return Some(node);
    }

    if cursor.node().kind() == "array" {
        return match path[0].parse() {
            Ok(idx) => {
                if let Some(node) = node.named_child(idx) {
                    return find_node(node, json, &path[1..]);
                }
                None
            }
            Err(_) => None,
        };
    }

    for child in node.named_children(&mut cursor) {
        if child.kind() != "pair" {
            continue;
        }

        let key = child.child_by_field_name("key");
        let Some(key) = key else {
            continue;
        };
        let key = key.utf8_text(json.as_bytes()).ok();
        let Some(key) = key else {
            continue;
        };

        if key.trim_matches('"') == path[0] {
            let value = child.child_by_field_name("value")?;
            if path.len() == 1 {
                return Some(value);
            }
            return find_node(value, json, &path[1..]);
        }
    }
    None
}

fn map_to_validation_error(
    error: (String, String),
    json: &str,
    parser: &mut Parser,
) -> ValidationError {
    let err_path = error.0;
    let err = error.1;

    if let Some(tree) = parser.parse(json, None)
        && let Some(root) = tree.root_node().child(0)
    {
        let path_parts = err_path
            .split('/')
            .filter(|&part| !part.is_empty())
            .collect::<Vec<&str>>();

        let Some(node) = find_node(root, json, &path_parts) else {
            return ValidationError::Error {
                message: err,
                path: err_path,
            };
        };

        let start = node.start_byte();
        let line_start = json[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);

        ValidationError::PosError {
            line: node.start_position().row + 1,
            column: json[line_start..start].chars().count() + 1,
            message: err,
            path: err_path,
        }
    } else {
        ValidationError::Error {
            message: err,
            path: err_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SchemaType;
    use crate::validation::validate;

    #[test]
    fn test_should_validate_mtb_patient_record() {
        const INPUT: &str = include_str!("../../test/mtb-patient-record.json");
        let validation_errors =
            validate(INPUT, SchemaType::MTB).expect("Validation should not end in error result");
        assert!(validation_errors.is_empty());
    }

    #[test]
    fn test_should_validate_rd_patient_record() {
        const INPUT: &str = include_str!("../../test/rd-patient-record.json");
        let validation_errors =
            validate(INPUT, SchemaType::RD).expect("Validation should not end in error result");
        assert!(validation_errors.is_empty());
    }
}
