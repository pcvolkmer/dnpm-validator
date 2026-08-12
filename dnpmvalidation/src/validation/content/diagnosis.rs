use crate::{ValidationError, ValidationType};
use crate::validation::content::{validate_contains_oneof, validate_contains_valueof};

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    if validation_type == ValidationType::Grz {
        return Ok(vec![]);
    }

    if validation_type == ValidationType::Mtb {
        errors.append(&mut validate_contains_oneof(
            json,
            "tumor staging classification",
            &["tnmClassification", "otherClassifications"],
            "$.diagnoses[*].staging.history[*]",
        )?);

        errors.append(&mut validate_contains_valueof(
            json,
            "tumor staging classification value",
            &["tumor-free", "local", "metastasized"],
            "$.diagnoses[*].staging.history[*].otherClassifications[*].code",
        )?);
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_should_find_missing_tumor_staging_classification_errors() {
        let json = json!({
            "diagnoses": [{
                "staging": {
                    "history": [
                      // Valid: TNM and Other
                      {
                        "date": "2022-10-01",
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
                      },
                      // Valid: Other only
                      {
                        "date": "2022-11-01",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "otherClassifications": [
                          {
                            "code": "metastasized",
                            "display": "Metastasiert",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      },
                      // Valid: TNM only
                      {
                        "date": "2022-12-01",
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
                        }
                      },
                      // INVALID!
                      {
                        "date": "2023-11-19",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        }
                      }
                    ]
                  }
            }]
        })
            .to_string();

        let actual = crate::validation::content::validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Missing tumor staging classification: should contain one of tnmClassification, otherClassifications"
        );
    }

    #[test]
    fn test_should_find_invalid_tumor_staging_other_classification_value_errors() {
        let json = json!({
            "diagnoses": [{
                "staging": {
                    "history": [
                      {
                        "date": "2022-10-01",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "otherClassifications": [
                          {
                            "code": "metastasized",
                            "display": "Metastasiert",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      }, {
                        "date": "2022-10-01",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "otherClassifications": [
                          {
                            "code": "local",
                            "display": "Lokal",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      }, {
                        "date": "2022-10-01",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "otherClassifications": [
                          {
                            "code": "tumor-free",
                            "display": "Tumorfrei",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      }, {
                        "date": "2022-10-01",
                        "method": {
                          "code": "clinical",
                          "display": "Klinisch",
                          "system": "dnpm-dip/mtb/tumor-staging/method"
                        },
                        "otherClassifications": [
                          {
                            "code": "wrong!",
                            "display": "Fehlerhafter Wert!",
                            "system": "dnpm-dip/mtb/diagnosis/kds-tumor-spread"
                          }
                        ]
                      }
                    ]
                  }
            }]
        })
            .to_string();

        let actual = crate::validation::content::validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Invalid tumor staging classification value 'wrong!': should be one of tumor-free, local, metastasized"
        );
    }
}