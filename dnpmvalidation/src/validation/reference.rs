use crate::validation::{ValidationError, ValidationType, map_to_validation_error};
use jsonpath_rust::JsonPath;
use serde_json::Value;
use tree_sitter::Parser;

pub fn validate(
    json: &str,
    validation_type: ValidationType,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    errors.append(&mut validate_refs(
        json,
        "Patient",
        "$.patient.id",
        "$..patient.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Specimen",
        "$.specimens[*].id",
        "$..specimen.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "HistologyReport",
        "$.histologyReports[*].id",
        "$..histology.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Diagnosis",
        "$.diagnoses[*].id",
        "$..reason.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Diagnosis",
        "$.diagnoses[*].id",
        "$..tumorEntity.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Variant",
        "$.ngsReports[*]..id",
        "$..variant.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Recommendation",
        "$..medicationRecommendations[*].id",
        "$..recommendation.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Recommendation",
        "$..claims[*].id",
        "$..claimResponses[*].claim.id",
    )?);

    errors.append(&mut validate_refs(
        json,
        "Therapy",
        "$..systemicTherapies[*].history[*].id",
        "$..responses[*].therapy.id",
    )?);

    if validation_type == ValidationType::Mtb {
        errors.append(&mut validate_refs(
            json,
            "Recommendation",
            "$..medicationRecommendations[*].id",
            "$..basedOn.id",
        )?);
    } else if validation_type == ValidationType::Rd {
        errors.append(&mut validate_refs(
            json,
            "Recommendation",
            "$..therapyRecommendations[*].id",
            "$..basedOn.id",
        )?);
    }

    Ok(errors)
}

fn validate_refs(
    json: &str,
    item_type: &str,
    item_path: &str,
    ref_path: &str,
) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(json)?;

    let patient = value.query_with_path(item_path)?;
    let references = value.query_with_path(ref_path)?;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_json::LANGUAGE.into())?;

    let errors = references
        .iter()
        .filter(|query_ref| {
            patient
                .iter()
                .find(|ref_id| ref_id.val.eq(query_ref.val))
                .is_none()
        })
        .map(|id| {
            let path = id
                .path
                .replace("$", "")
                .replace("'", "")
                .replace("]", "")
                .replace("[", "/");
            let value = id.val.to_string();
            (path, value)
        })
        .map(|(err_path, value)| {
            map_to_validation_error(
                (
                    err_path,
                    format!(
                        "Invalid reference to {} {}",
                        item_type,
                        value.replace("\"", "'")
                    ),
                ),
                json,
                &mut parser,
            )
        })
        .collect::<Vec<_>>();

    Ok(errors)
}
