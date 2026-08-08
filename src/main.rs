mod validation;

use crate::validation::{ValidationError, validate};
use clap::{Parser as ClapParser, ValueEnum};
use console::style;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(ClapParser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    #[arg(help = "The file to be checked")]
    pub file: PathBuf,

    #[arg(long = "type", default_value = "mtb", help = "The schema to be used")]
    pub schema: SchemaType,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum SchemaType {
    MTB,
    RD,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let file = fs::read_to_string(cli.file)?;

    print!(
        "Validation using {} schema: ",
        style(match cli.schema {
            SchemaType::MTB => "MTB",
            SchemaType::RD => "RD",
        })
        .underlined()
    );

    let errors = validate(&file, cli.schema).unwrap_or_default();

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

    errors.iter().for_each(|err| match err {
        ValidationError::Error { message, path } => println!(
            "{} {} {}",
            style("Validation error").red(),
            message,
            if path.is_empty() {
                String::new()
            } else {
                format!("at '{}'", path)
            }
        ),
        ValidationError::PosError {
            line,
            column,
            message,
            path,
        } => println!(
            "{} {} {}",
            style(format!("Validation error [{:>4}:{:>4}]", line, column)).red(),
            message,
            if path.is_empty() {
                String::new()
            } else {
                format!("at '{}'", path)
            }
        ),
    });

    exit(1)
}
