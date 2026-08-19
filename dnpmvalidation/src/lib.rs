pub use crate::validation::{Severity, ValidationError, ValidationType, pretty_print, validate};

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
        pub start: Position,
        pub end: Position,
        pub message: String,
        pub path: String,
        pub severity: Severity,
    }

    struct Position {
        pub line: usize,
        pub column: usize,
    }

    enum Severity {
        Error,
        Warning,
        Information,
    }

    extern "Rust" {
        #[cxx_name = "validate"]
        fn validate_cxx(
            json: String,
            validation_type: ValidationType,
            report_severity: Severity,
        ) -> Vec<ValidationError>;

        #[cxx_name = "pretty_print"]
        fn pretty_print_cxx(json: String) -> String;
    }
}

impl Default for ffi::Position {
    fn default() -> Self {
        Self { line: 0, column: 0 }
    }
}

pub fn validate_cxx(
    json: String,
    validation_type: ffi::ValidationType,
    report_severity: ffi::Severity,
) -> Vec<ffi::ValidationError> {
    match validate(
        json.as_str(),
        match validation_type {
            ffi::ValidationType::Mtb => ValidationType::Mtb,
            ffi::ValidationType::Rd => ValidationType::Rd,
            ffi::ValidationType::Grz => ValidationType::Grz,
            _ => ValidationType::Mtb,
        },
        match report_severity {
            ffi::Severity::Error => &Severity::Error,
            ffi::Severity::Warning => &Severity::Warning,
            _ => &Severity::Information,
        },
    ) {
        Ok(validations) => validations
            .into_iter()
            .map(|validation| ffi::ValidationError {
                message: validation.message,
                path: validation.path,
                start: ffi::Position {
                    line: validation.start.line,
                    column: validation.start.column,
                },
                end: ffi::Position {
                    line: validation.end.line,
                    column: validation.end.column,
                },
                severity: match validation.severity {
                    Severity::Error => ffi::Severity::Error,
                    Severity::Warning => ffi::Severity::Warning,
                    Severity::Information => ffi::Severity::Information,
                },
            })
            .collect(),
        Err(err) => vec![ffi::ValidationError {
            message: err.to_string(),
            path: "".to_string(),
            start: ffi::Position::default(),
            end: ffi::Position::default(),
            severity: ffi::Severity::Error,
        }],
    }
}

pub fn pretty_print_cxx(json: String) -> String {
    pretty_print(json.as_str()).unwrap_or_default()
}
