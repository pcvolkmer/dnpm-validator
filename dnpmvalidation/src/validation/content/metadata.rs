use crate::validation::Severity;
use crate::validation::content::{validate_contains_valueof, validate_regex,
};
use crate::{ValidationError, ValidationType};
use regex::Regex;

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    if validation_type == ValidationType::Grz {
        return Ok(vec![]);
    }

    errors.append(&mut validate_regex(
        json,
        "TransferTAN",
        &Regex::new(r"^[a-fA-F0-9]{64}$").expect("Valid regex expected"),
        "$.metadata.transferTAN",
        &Severity::Error,
    )?);

    errors.append(&mut validate_contains_valueof(
        json,
        "reason missing broad consent (since 2026-06-01)",
        &[
            "patient-inability",
            "patient-refusal",
            "consent-not-returned",
            "other-patient-reason",
        ],
        "$.metadata.reasonResearchConsentMissing",
        &Severity::Error,
    )?);

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("012345", false)]
    #[case("", false)]
    #[case("xyz123", false)]
    #[case(
        "2c5e378b5a5a45d59e2ff67214a6851a2c5e378b5a5a45d59e2ff67214a6851a",
        true
    )]
    fn test_should_find_transfertan_errors(#[case] transfer_tan: &str, #[case] is_valid: bool) {
        let json = json!({
            "metadata": {
                "type": "followup",
                "transferTAN": transfer_tan,
                "reasonResearchConsentMissing": "other-patient-reason"
            }
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");

        if is_valid {
            assert!(actual.is_empty());
        } else {
            assert_eq!(actual.len(), 1);
            assert_eq!(
                actual[0].message,
                format!("Invalid TransferTAN '{transfer_tan}'")
            );
        }
    }

    #[rstest]
    #[case("patient-inability", true)]
    #[case("patient-refusal", true)]
    #[case("consent-not-returned", true)]
    #[case("other-patient-reason", true)]
    #[case("technical-issues", false)]
    #[case("organizational-issues", false)]
    fn test_should_find_reason_research_consent_missing_errors(#[case] reason: &str, #[case] is_valid: bool) {
        let json = json!({
            "metadata": {
                "type": "initial",
                "transferTAN": "2c5e378b5a5a45d59e2ff67214a6851a2c5e378b5a5a45d59e2ff67214a6851a",
                "reasonResearchConsentMissing": reason
            }
        })
            .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");

        if is_valid {
            assert!(actual.is_empty());
        } else {
            assert_eq!(actual.len(), 1);
            assert_eq!(
                actual[0].message,
                format!("Invalid reason missing broad consent (since 2026-06-01) '{reason}': must be one of patient-inability, patient-refusal, consent-not-returned, other-patient-reason")
            );
        }
    }
}
