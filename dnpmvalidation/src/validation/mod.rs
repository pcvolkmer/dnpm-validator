mod content;
mod reference;
pub mod schema;

use jsonpath_rust::query::QueryRef;
use serde_json::Value;
use tree_sitter::{Node, Parser};

#[allow(unused)]
#[derive(Debug)]
pub struct ValidationError {
    pub start: Position,
    pub end: Position,
    pub message: String,
    pub path: String,
    pub severity: Severity,
}

#[derive(Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    Error,
    Warning,
    Information,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Error
    }
}

impl Position {
    pub fn empty() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationType {
    Mtb,
    Rd,
    Grz,
}

pub fn validate(
    json: &str,
    schema: ValidationType,
    report_severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    if serde_json::from_str::<Value>(json).is_err() {
        return Ok(vec![ValidationError {
            start: Position::empty(),
            end: Position::empty(),
            message: "Invalid JSON file".to_string(),
            path: String::new(),
            severity: Severity::Error,
        }]);
    }

    let mut errors = schema::validate(json, schema)?;
    errors.append(&mut reference::validate(json, schema)?);
    errors.append(&mut content::validate(json, schema)?);

    errors.retain(|item| &item.severity <= report_severity);

    errors.sort_by_key(|ve| ve.start.column);
    errors.sort_by_key(|ve| ve.start.line);
    errors.sort_by_key(|ve| ve.severity);

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
    severity: &Severity,
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
            return ValidationError {
                start: Position::empty(),
                end: Position::empty(),
                message: err,
                path: err_path,
                severity: severity.clone(),
            };
        };

        let start = node.start_byte();
        let end = node.end_byte();
        let line_start = json[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = json[..end].rfind('\n').map(|i| i + 1).unwrap_or(0);

        ValidationError {
            start: Position::new(
                node.start_position().row + 1,
                json[line_start..start].chars().count() + 1,
            ),
            end: Position::new(
                node.end_position().row + 1,
                json[line_end..end].chars().count() + 1,
            ),
            message: err,
            path: err_path,
            severity: severity.clone(),
        }
    } else {
        ValidationError {
            start: Position::empty(),
            end: Position::empty(),
            message: err,
            path: err_path,
            severity: severity.clone(),
        }
    }
}

fn map_query_ref(query_ref: &QueryRef<Value>) -> (String, String) {
    let path = query_ref
        .path
        .replace("$", "")
        .replace("'", "")
        .replace("]", "")
        .replace("[", "/");
    let value = query_ref.val.as_str().unwrap_or_default().to_string();
    (path, value)
}

#[cfg(test)]
mod tests {
    use crate::validation::{Severity, ValidationType, validate};

    #[test]
    fn test_should_validate_mtb_patient_record() {
        const INPUT: &str = include_str!("../../../test/mtb-patient-record.json");
        let validation_errors = validate(INPUT, ValidationType::Mtb, &Severity::Error)
            .expect("Validation should not end in error result");
        assert!(validation_errors.is_empty());
    }

    #[test]
    fn test_should_validate_rd_patient_record() {
        const INPUT: &str = include_str!("../../../test/rd-patient-record.json");
        let validation_errors = validate(INPUT, ValidationType::Rd, &Severity::Error)
            .expect("Validation should not end in error result");
        assert!(validation_errors.is_empty());
    }
}
