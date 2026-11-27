use crate::validation::Severity;
use crate::validation::content::{
    validate_contains_oneof, validate_contains_valueof, validate_fancy_regex,
};
use crate::{ValidationError, ValidationType};
use fancy_regex::Regex;

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    if validation_type == ValidationType::Grz {
        return Ok(vec![]);
    }

    errors.append(&mut validate_contains_oneof(
        json,
        "CarePlan BoardType",
        &["boardType"],
        "$.carePlans",
        &Severity::Error,
    )?);

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_should_find_missing_board_type() {
        let json = json!({
        "carePlans": [
          {
            "id": "7242b8d5-7295-467c-88eb-6980b52c03d2",
            "patient": {
              "id": "951d12c5-9d52-472e-96cc-1eb3576e158c",
              "type": "Patient"
            },
            "reason": {
              "id": "ea137aa5-199d-4894-b6a9-fd3dc08327cd",
              "type": "MTBDiagnosis"
            },
            "issuedOn": "2026-11-13",
            "boardType": {
              "code": "indication-board",
              "display": "Indikations-Board",
              "system": "dnpm-dip/careplan/board-type"
            }
            }]
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Missing CarePlan BoardType: must contain 'boardType'"
        );
    }
}
