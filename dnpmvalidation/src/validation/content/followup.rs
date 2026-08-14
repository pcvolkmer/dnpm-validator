use crate::validation::Severity;
use crate::validation::content::{validate_contains_oneof, validate_min_items};
use crate::{ValidationError, ValidationType};
use jsonpath_rust::JsonPath;
use serde_json::Value;

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    if validation_type == ValidationType::Grz {
        return Ok(vec![]);
    }

    if validation_type == ValidationType::Mtb
        && serde_json::from_str::<Value>(json)?
            .query_with_path("$.metadata.type")?
            .iter()
            .filter_map(|query_ref| query_ref.val.as_str())
            .filter(|&s| s == "followup")
            .count()
            > 0
    {
        errors.append(&mut validate_contains_oneof(
            json,
            "followUps",
            &["followUps"],
            "$",
            &Severity::Error,
        )?);

        errors.append(&mut validate_min_items(
            json,
            "followUps",
            1,
            "$.followUps",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_oneof(
            json,
            "claims (follow-up)",
            &["claims"],
            "$",
            &Severity::Warning,
        )?);

        errors.append(&mut validate_contains_oneof(
            json,
            "claim responses (follow-up)",
            &["claimResponses"],
            "$",
            &Severity::Warning,
        )?);

        errors.append(&mut validate_contains_oneof(
            json,
            "responses (follow-up)",
            &["responses"],
            "$",
            &Severity::Warning,
        )?);
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_should_find_missing_follow_ups() {
        let json = json!({
            "metadata": {
                "type": "followup",
                "transferTAN": "2320c885aae5433e2320c885aae5433e2320c885aae5433e",
                "modelProjectConsent": {
                  "version": "Version 1",
                  "provisions": [
                    {
                      "date": "2026-01-01",
                      "purpose": "sequencing",
                      "type": "permit"
                    }
                  ]
                }
            }
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 4);
        assert_eq!(
            actual[0].message,
            "Missing followUps: must contain 'followUps'"
        );
        assert_eq!(
            actual[1].message,
            "Missing claims (follow-up): should contain 'claims'"
        );
        assert_eq!(
            actual[2].message,
            "Missing claim responses (follow-up): should contain 'claimResponses'"
        );
        assert_eq!(
            actual[3].message,
            "Missing responses (follow-up): should contain 'responses'"
        );
    }

    #[test]
    fn test_should_find_empty_follow_ups() {
        let json = json!({
            "metadata": {
                "type": "followup",
                "transferTAN": "2320c885aae5433e2320c885aae5433e2320c885aae5433e",
                "modelProjectConsent": {
                  "version": "Version 1",
                  "provisions": [
                    {
                      "date": "2026-01-01",
                      "purpose": "sequencing",
                      "type": "permit"
                    }
                  ]
                }
            },
            "followUps": []
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 4);
        assert_eq!(
            actual[0].message,
            "Invalid followUps: must contain some items"
        );
        assert_eq!(
            actual[1].message,
            "Missing claims (follow-up): should contain 'claims'"
        );
        assert_eq!(
            actual[2].message,
            "Missing claim responses (follow-up): should contain 'claimResponses'"
        );
        assert_eq!(
            actual[3].message,
            "Missing responses (follow-up): should contain 'responses'"
        );
    }
}
