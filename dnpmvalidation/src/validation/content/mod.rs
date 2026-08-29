mod diagnosis;
mod followup;
mod metadata;

use crate::validation::{Severity, map_query_ref, map_to_validation_error};
use crate::{ValidationError, ValidationType};
use jsonpath_rust::JsonPath;
use regex::Regex;
use serde_json::Value;
use tree_sitter::Parser;

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
        "MV-Metadata",
        &["metadata"],
        "$",
        &Severity::Warning,
    )?);

    errors.append(&mut validate_regex(
        json,
        "ATC code",
        &Regex::new("^[ABCDGHJLMNPRSV][0-2][0-9]([A-Z]([A-Z](\\d{2})?)?)?$")
            .expect("Valid regex expected"),
        "$..medication[?(@.system == 'http://fhir.de/CodeSystem/bfarm/atc')].code",
        &Severity::Error,
    )?);

    errors.append(&mut validate_date(
        json,
        "$..period.start",
        &Severity::Error,
    )?);

    errors.append(&mut validate_date(json, "$..period.end", &Severity::Error)?);

    errors.append(&mut validate_date(
        json,
        "$..effectiveDate",
        &Severity::Error,
    )?);

    errors.append(&mut validate_date(json, "$..date", &Severity::Error)?);

    errors.append(&mut validate_date(json, "$..issuedOn", &Severity::Error)?);

    errors.append(&mut validate_date(json, "$..recordedOn", &Severity::Error)?);

    errors.append(&mut validate_period(json, "$..period", &Severity::Error)?);

    if validation_type == ValidationType::Mtb {
        errors.append(&mut metadata::validate(json, validation_type)?);
        errors.append(&mut diagnosis::validate(json, validation_type)?);
        errors.append(&mut followup::validate(json, validation_type)?);
    }

    Ok(errors)
}

fn validate_regex(
    json: &str,
    name: &str,
    regex: &Regex,
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| !regex.is_match(item.val.as_str().unwrap_or_default()))
        .map(map_query_ref)
        .map(|(err_path, value)| {
            map_to_validation_error(
                (err_path, format!("Invalid {name} '{value}'")),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_fancy_regex(
    json: &str,
    name: &str,
    regex: &fancy_regex::Regex,
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            !regex
                .is_match(&item.val.as_str().unwrap_or_default())
                .unwrap_or_default()
        })
        .map(map_query_ref)
        .map(|(err_path, value)| {
            map_to_validation_error(
                (err_path, format!("Invalid {name} '{value}'")),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_min_items(
    json: &str,
    name: &str,
    min_items: usize,
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            let array = item.val.as_array();
            array.is_some() && array.expect("Valid array").iter().count() < min_items
        })
        .map(map_query_ref)
        .map(|(err_path, _)| {
            map_to_validation_error(
                (
                    err_path,
                    format!(
                        "Invalid {name}: {} contain {}",
                        match severity {
                            Severity::Error => "must",
                            Severity::Warning | Severity::Information => "should",
                        },
                        if min_items == 1 {
                            "some items".into()
                        } else {
                            format!("at least {min_items} items")
                        }
                    ),
                ),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_contains_oneof(
    json: &str,
    name: &str,
    keys: &[&str],
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            let Some(obj) = item.val.as_object() else {
                return true;
            };
            !keys.iter().any(|&subpath| obj.contains_key(subpath))
        })
        .map(map_query_ref)
        .map(|(err_path, _)| {
            map_to_validation_error(
                (
                    err_path,
                    if keys.len() > 1 {
                        format!(
                            "Missing {name}: {} contain one of {}",
                            match severity {
                                Severity::Error => "must",
                                Severity::Warning | Severity::Information => "should",
                            },
                            keys.join(", ")
                        )
                    } else {
                        format!(
                            "Missing {name}: {} contain '{}'",
                            match severity {
                                Severity::Error => "must",
                                Severity::Warning | Severity::Information => "should",
                            },
                            keys.first().unwrap_or(&"")
                        )
                    },
                ),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_contains_valueof(
    json: &str,
    name: &str,
    values: &[&str],
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            let Some(value) = item.val.as_str() else {
                return true;
            };
            !values.contains(&value)
        })
        .map(map_query_ref)
        .map(|(err_path, value)| {
            map_to_validation_error(
                (
                    err_path,
                    format!(
                        "Invalid {name} '{value}': {} be one of {}",
                        match severity {
                            Severity::Error => "must",
                            Severity::Warning | Severity::Information => "should",
                        },
                        values.join(", ")
                    ),
                ),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_period(
    json: &str,
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let date_regex = Regex::new(r"\d{4}-\d{2}-\d{2}").expect("Invalid regex");

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            let Some(value) = item.val.as_object() else {
                return true;
            };

            let period_start = if value.contains_key("start")
                && let Some(period_start) = value["start"].as_str()
                && date_regex.is_match(period_start)
            {
                period_start
            } else {
                return true;
            };

            if value.contains_key("end")
                && let Some(period_end) = value["end"].as_str()
            {
                return !date_regex.is_match(period_end) || period_start > period_end;
            }

            false
        })
        .map(map_query_ref)
        .map(|(err_path, _)| {
            map_to_validation_error(
                (err_path, "Invalid period".to_string()),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

fn validate_date(
    json: &str,
    path: &str,
    severity: &Severity,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$")?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| {
            let Some(date_str) = item.val.as_str() else {
                return true;
            };

            if !re.is_match(date_str) {
                return true;
            }

            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err()
        })
        .map(map_query_ref)
        .map(|(err_path, value)| {
            map_to_validation_error(
                (err_path, format!("Invalid date '{value}'")),
                json,
                &mut parser,
                severity,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[test]
    fn test_should_find_atc_code_errors() {
        let json = json!({
            "carePlans": [{
                "guidelineTherapies": [{
                  "id": "1",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testonimib A",
                      "display": "Testonimib A",
                      "system": "http://fhir.de/CodeSystem/bfarm/atc",
                      "version": "2025"
                    }
                  ],
                }, {
                  "id": "2",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testanimab",
                      "display": "Testanimab",
                      "system": "unknown"
                    }
                  ],
                }],
                "medicationRecommendations": [{
                  "id": "1",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testonimib B",
                      "display": "Testonimib B",
                      "system": "http://fhir.de/CodeSystem/bfarm/atc",
                      "version": "2025"
                    }
                  ],
                }, {
                  "id": "2",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testanimab",
                      "display": "Testanimab",
                      "system": "unknown"
                    }
                  ],
                }],
                "systemicTherapies": [{
                  "id": "1",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testonimib C",
                      "display": "Testonimib C",
                      "system": "http://fhir.de/CodeSystem/bfarm/atc",
                      "version": "2025"
                    }
                  ],
                }, {
                  "id": "2",
                  "patient": {
                    "id": "2320c885-aae5-433e-847b-d8ae62186530",
                    "type": "Patient"
                  },
                  "medication": [
                    {
                      "code": "Testanimab",
                      "display": "Testanimab",
                      "system": "unknown"
                    }
                  ],
                }],
            }]
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 4);
        assert_eq!(
            actual[0].message,
            "Missing MV-Metadata: should contain 'metadata'"
        );
        assert_eq!(actual[1].message, "Invalid ATC code 'Testonimib A'");
        assert_eq!(actual[2].message, "Invalid ATC code 'Testonimib B'");
        assert_eq!(actual[3].message, "Invalid ATC code 'Testonimib C'");
    }

    #[rstest]
    #[case("2025-08-23", false)]
    #[case("2025-08-1", true)]
    #[case("2025-08", true)]
    #[case("", true)]
    fn test_should_find_date_errors(#[case] date: &str, #[case] has_error: bool) {
        let json = json!({
            "diagnoses": [{
                "staging": {
                    "history": [
                      {
                        // injected date
                        "date": date,
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "tnmClassification": {
                          "tumor": {
                            "code": "T2",
                            "system": "UICC"
                          },
                          "nodes": {
                            "code": "N0",
                            "system": "UICC"
                          },
                          "metastasis": {
                            "code": "MX",
                            "system": "UICC"
                          }
                        },
                        "otherClassifications": [
                          {
                            "code": "metastasized",
                            "display": "Metastasiert",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      }
                    ]
                  }
            }]
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), if has_error { 2 } else { 1 });
        assert_eq!(
            actual[0].message,
            "Missing MV-Metadata: should contain 'metadata'"
        );
        if has_error {
            assert_eq!(actual[1].message, format!("Invalid date '{date}'"));
        }
    }

    #[rstest]
    #[case("2025-05-20", "2025-05-20", false)]
    #[case("2025-05-21", "2025-05-20", true)]
    #[case("2025-05-20", "2025-05-21", false)]
    fn test_should_find_period_errors(
        #[case] period_start: &str,
        #[case] period_end: &str,
        #[case] has_error: bool,
    ) {
        let json = json!({
        "systemicTherapies": [{
          "history": [
            {
              "id": "0d30a28a-cdc5-48c5-8cc9-3120a6ff3383",
              "patient": {
                "id": "2320c885-aae5-433e-847b-d8ae62186530",
                "type": "Patient"
              },
              "reason": {
                "id": "871399bf-f2c3-45ab-b7af-3958b17e1d34",
                "display": "Bösartige Neubildung: Milz",
                "type": "MTBDiagnosis"
              },
              "intent": {
                "code": "K",
                "display": "Kurativ",
                "system": "dnpm-dip/therapy/intent"
              },
              "category": {
                "code": "A",
                "display": "Adjuvant",
                "system": "dnpm-dip/therapy/category"
              },
              "basedOn": {
                "id": "82bae1f3-5642-4cdf-8b4c-28aaa4606ecd",
                "type": "MTBMedicationRecommendation"
              },
              "recordedOn": "2025-10-14",
              "status": {
                "code": "stopped",
                "display": "Abgebrochen",
                "system": "dnpm-dip/therapy/status"
              },
              "statusReason": {
                "code": "progression",
                "display": "Progression",
                "system": "dnpm-dip/therapy/status-reason"
              },
              "recommendationFulfillmentStatus": {
                "code": "partial",
                "display": "Partiell",
                "system": "dnpm-dip/therapy/recommendation-fulfillment-status"
              },
              "dosage": {
                "code": "under-50%",
                "display": "< 50 %",
                "system": "dnpm-dip/therapy/dosage-density"
              },
              "period": {
                "start": period_start,
                "end": period_end
              },
              "medication": [
                {
                  "code": "L01XX74",
                  "display": "Belzutifan",
                  "system": "http://fhir.de/CodeSystem/bfarm/atc",
                  "version": "2025"
                }
              ],
              "notes": [
                "Notes on the therapy..."
              ]
            }
          ]
        }]
        })
        .to_string();

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), if has_error { 2 } else { 1 });
        assert_eq!(
            actual[0].message,
            "Missing MV-Metadata: should contain 'metadata'"
        );
        if has_error {
            assert_eq!(actual[1].message, "Invalid period");
        }
    }
}
