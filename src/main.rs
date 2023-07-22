use clap::Parser;
use program::Program;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
mod util;
use util::shell;

use anyhow::{Context, Result};

mod program;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Read from <FILE_NAME>
    #[arg(short, long)]
    file_name: String,
    /// Only parse the code and exit
    #[arg(long, default_value_t = false)]
    only_parse: bool,
    /// Enable features from <FEATURES> (Features still need to be enable in build step)
    #[arg(long, value_delimiter = ',', use_value_delimiter = true)]
    features: Vec<String>,
}

fn main() -> Result<()> {
    let mut shell = shell::Shell::new();
    let args = Args::parse();
    let vec_ast = program::parser::Ast::parse_code(
        fs::read_to_string(&args.file_name)
            .with_context(|| {
                let _ = shell.error("File error");
                format!("Failed to read file from {}", args.file_name)
            })?
            .replace("\r\n", "")
            .as_str(),
    )
    .with_context(|| {
        let _ = shell.error("Parse error");
        format!("Failed to parse file from {}", args.file_name)
    })?;
    if args.only_parse {
        println!("{:#?}", vec_ast);
        exit(0);
    };
    let mut features_list = vec!["return".to_owned(), "print".to_owned(), "input".to_owned(), "split".to_owned()];
    for feature in &args.features {
        if !features_list.contains(&feature) {
            features_list.push(feature.to_string());
        }
    }
    let mut featureliststr = "".to_string();
    for feature in &features_list {
        featureliststr = featureliststr + "`" + &feature + "`" + " ";
    }
    let mut program: Program = Program {
        commands: vec_ast,
        current_line: 0,
        variable: HashMap::new(),
        function: HashMap::new(),
        std_commands: features_list,
    };
    shell
        .status("Running", "with feature ".to_string() + &featureliststr)
        .unwrap();
    let output = program.run_loop(&mut Vec::new(), &mut shell);
    if let Ok(returnval) = output {
        match returnval {
            program::Data::Number(e) => exit(e.round().to_string().parse().unwrap()),
            _ => unimplemented!(),
        }
    } else {
        shell.error("We fucked up")
    }
}
