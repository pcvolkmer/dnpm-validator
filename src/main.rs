use clap::{Parser as ClapParser, ValueEnum};
use console::style;
use dnpmvalidation::{ValidationType, validate};
use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(ClapParser)]
#[command(author, version, about)]
#[command(help_template = "{name} {version}\n{about}\n\n{usage-heading} {usage}\n\n{all-args}")]
#[command(arg_required_else_help(true))]
pub struct Cli {
    #[arg(help = "The file to be validated")]
    pub file: PathBuf,

    #[arg(
        long = "type",
        default_value = "mtb",
        help = "The schema to be used for validation"
    )]
    pub schema: SchemaType,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum SchemaType {
    Mtb,
    Rd,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let file = fs::read_to_string(cli.file)?;

    print!(
        "Validation using {} schema: ",
        style(match cli.schema {
            SchemaType::Mtb => "MTB",
            SchemaType::Rd => "RD",
        })
        .underlined()
    );

    let errors = validate(
        &file,
        match cli.schema {
            SchemaType::Mtb => ValidationType::Mtb,
            SchemaType::Rd => ValidationType::Rd,
        },
    )
    .unwrap_or_default();

    if errors.is_empty() {
        println!("{}", style("No validation errors found").green().bold());
        return Ok(());
    } else if errors.len() == 1 {
        println!("{}\n", style("Found 1 validation error").red().bold());
    } else {
        println!(
            "{}\n",
            style(format!("Found {} validation errors", errors.len()))
                .red()
                .bold()
        );
    }

    errors.iter().for_each(|err| {
        println!(
            "{} {} {}",
            style(format!(
                "🔥 Validation error {:<11}",
                format!("[{}:{}]", err.start.line, err.start.column)
            ))
            .red(),
            err.message,
            if err.path.is_empty() {
                String::new()
            } else {
                format!("at '{}'", err.path)
            }
        )
    });

    exit(1)
}
