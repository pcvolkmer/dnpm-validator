use crate::validation::{map_query_ref, map_to_validation_error};
use crate::{ValidationError, ValidationType};
use jsonpath_rust::JsonPath;
use regex::Regex;
use serde_json::Value;
use tree_sitter::Parser;

pub fn validate(
    json: &str,
    _: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    errors.append(&mut validate_regex(
        json,
        "ATC code",
        &Regex::new("^[ABCDGHJLMNPRSV][0-2][1-9]([A-Z]([A-Z](\\d{2})?)?)?$")
            .expect("Valid regex expected"),
        "$..medication[?(@.system == 'http://fhir.de/CodeSystem/bfarm/atc')].code",
    )?);

    Ok(errors)
}

fn validate_regex(
    json: &str,
    name: &str,
    regex: &Regex,
    path: &str,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let mut errors = Vec::new();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let mut value = value
        .query_with_path(path)?
        .iter()
        .filter(|item| !regex.is_match(&item.val.as_str().unwrap_or_default()))
        .map(map_query_ref)
        .map(|(err_path, value)| {
            map_to_validation_error(
                (err_path, format!("Invalid {name} '{value}'")),
                json,
                &mut parser,
            )
        })
        .collect::<Vec<_>>();

    errors.append(&mut value);

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0].message, "Invalid ATC code 'Testonimib A'");
        assert_eq!(actual[1].message, "Invalid ATC code 'Testonimib B'");
        assert_eq!(actual[2].message, "Invalid ATC code 'Testonimib C'");
    }
}
