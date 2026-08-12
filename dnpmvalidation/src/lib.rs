pub use crate::validation::{ValidationError, ValidationType, validate};
use ffi::ValidationType as FfiValidationType;

mod validation;

#[cxx::bridge(namespace = "dnpmvalidation")]
mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ValidationType {
        Mtb,
        Rd,
        Grz,
    }

    struct ValidationError {
        pub startLine: usize,
        pub startColumn: usize,
        pub endLine: usize,
        pub endColumn: usize,
        pub message: String,
    }

    extern "Rust" {
        #[cxx_name = "validate"]
        fn validate_cxx(json: String, validation_type: ValidationType) -> Vec<ValidationError>;
    }
}

pub fn validate_cxx(json: String, validation_type: FfiValidationType) -> Vec<ffi::ValidationError> {
    match validate(
        json.as_str(),
        match validation_type {
            FfiValidationType::Mtb => ValidationType::Mtb,
            FfiValidationType::Rd => ValidationType::Rd,
            FfiValidationType::Grz => ValidationType::Grz,
            _ => ValidationType::Mtb,
        },
    ) {
        Ok(validations) => validations
            .into_iter()
            .map(|validation| ffi::ValidationError {
                message: validation.message,
                startLine: validation.start.line,
                startColumn: validation.start.column,
                endLine: validation.end.line,
                endColumn: validation.end.column,
            })
            .collect(),
        Err(err) => vec![ffi::ValidationError {
            message: err.to_string(),
            startLine: 0,
            startColumn: 0,
            endLine: 0,
            endColumn: 0,
        }],
    }
}
