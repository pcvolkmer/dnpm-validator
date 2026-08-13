use crate::validation::Severity;
use crate::validation::content::{validate_contains_oneof, validate_contains_valueof};
use crate::{ValidationError, ValidationType};

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    if validation_type == ValidationType::Grz {
        return Ok(vec![]);
    }

    if validation_type == ValidationType::Mtb {
        errors.append(&mut validate_contains_valueof(
            json,
            "tumor grading code",
            &["0", "1", "2", "3", "4", "5", "X", "L", "M", "H", "B", "U", "T"],
            "$.diagnoses[*].grading.history[*].codes[?(@.system == 'https://www.basisdatensatz.de/feld/161/grading')].code",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_valueof(
            json,
            "tumor grading code",
            &["I", "II", "III", "IV"],
            "$.diagnoses[*].grading.history[*].codes[?(@.system == 'dnpm-dip/mtb/who-grading-cns-tumors' && @.version == '2016')].code",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_valueof(
            json,
            "tumor grading code",
            &["1", "2", "3", "4"],
            "$.diagnoses[*].grading.history[*].codes[?(@.system == 'dnpm-dip/mtb/who-grading-cns-tumors' && @.version == '2021')].code",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_valueof(
            json,
            "tumor grading version",
            &["2016", "2021"],
            "$.diagnoses[*].grading.history[*].codes[*].version",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_oneof(
            json,
            "tumor staging classification",
            &["tnmClassification", "otherClassifications"],
            "$.diagnoses[*].staging.history[*]",
            &Severity::Error,
        )?);

        errors.append(&mut validate_contains_valueof(
            json,
            "tumor staging classification value",
            &["tumor-free", "local", "metastasized"],
            "$.diagnoses[*].staging.history[*].otherClassifications[*].code",
            &Severity::Error,
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

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Missing tumor staging classification: must contain one of tnmClassification, otherClassifications"
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

        let actual = validate(&json, ValidationType::Mtb);

        assert!(actual.is_ok());

        let actual = actual.expect("available validation results");
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Invalid tumor staging classification value 'wrong!': must be one of tumor-free, local, metastasized"
        );
    }

    #[test]
    fn test_should_find_invalid_tumor_grading_cns_codes() {
        let json = json!({
            "diagnoses": [{
                "grading": {
                    "history": [
                      {
                        "date": "2022-11-19",
                        "codes": [
                          {
                            "code": "B",
                            "display": "B = Borderline",
                            "system": "https://www.basisdatensatz.de/feld/161/grading"
                          },
                          {
                            "code": "3",
                            "display": "Anaplastic astrocytoma",
                            "system": "dnpm-dip/mtb/who-grading-cns-tumors",
                            "version": "2021"
                          }
                        ]
                      },
                      // Invalid code for version 2016
                      {
                        "date": "2022-11-19",
                        "codes": [
                          {
                            "code": "B",
                            "display": "B = Borderline",
                            "system": "https://www.basisdatensatz.de/feld/161/grading"
                          },
                          {
                            "code": "3",
                            "display": "Anaplastic astrocytoma",
                            "system": "dnpm-dip/mtb/who-grading-cns-tumors",
                            "version": "2016"
                          }
                        ]
                      },
                      // Invalid code for version 2021
                      {
                        "date": "2022-11-19",
                        "codes": [
                          {
                            "code": "B",
                            "display": "B = Borderline",
                            "system": "https://www.basisdatensatz.de/feld/161/grading"
                          },
                          {
                            "code": "IV",
                            "display": "Anaplastic astrocytoma",
                            "system": "dnpm-dip/mtb/who-grading-cns-tumors",
                            "version": "2021"
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
        assert_eq!(actual.len(), 2);
        assert_eq!(
            actual[0].message,
            "Invalid tumor grading code '3': must be one of I, II, III, IV"
        );
        assert_eq!(
            actual[1].message,
            "Invalid tumor grading code 'IV': must be one of 1, 2, 3, 4"
        );
    }

    #[test]
    fn test_should_find_invalid_tumor_grading_cns_versions() {
        let json = json!({
            "diagnoses": [{
                "grading": {
                    "history": [
                      {
                        "date": "2022-11-19",
                        "codes": [
                          {
                            "code": "B",
                            "display": "B = Borderline",
                            "system": "https://www.basisdatensatz.de/feld/161/grading"
                          },
                          {
                            "code": "3",
                            "display": "Anaplastic astrocytoma",
                            "system": "dnpm-dip/mtb/who-grading-cns-tumors",
                            "version": "2021"
                          }
                        ]
                      },
                      // Invalid version '2026'
                      {
                        "date": "2022-11-19",
                        "codes": [
                          {
                            "code": "B",
                            "display": "B = Borderline",
                            "system": "https://www.basisdatensatz.de/feld/161/grading"
                          },
                          {
                            "code": "3",
                            "display": "Anaplastic astrocytoma",
                            "system": "dnpm-dip/mtb/who-grading-cns-tumors",
                            "version": "2026"
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
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].message,
            "Invalid tumor grading version '2026': must be one of 2016, 2021"
        );
    }
}
