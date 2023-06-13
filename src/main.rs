#![feature(test)]

use clap::Parser;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::process::exit;
use std::{fs, process};

use program::Program;

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

fn main() {
    let args = Args::parse();
    let vec_ast = match program::parser::Ast::parse_code(
        fs::read_to_string(args.file_name)
            .unwrap()
            .replace("\r\n", "")
            .as_str(),
    ) {
        Ok(ast) => ast,
        Err(_e) => {
            eprintln!("Failed to read file");
            process::exit(1);
        }
    };
    if args.only_parse {
        println!("{:#?}", vec_ast);
        exit(0);
    };
    let mut features_list = vec!["return".to_owned(), "print".to_owned()];
    for feature in &args.features {
        if !features_list.contains(&feature) {
            features_list.push(feature.to_string());
        }
    }
    let mut program: Program = Program {
        commands: vec_ast,
        current_line: 0,
        panic: false,
        variable: HashMap::new(),
        function: HashMap::new(),
        std_commands: features_list,
        returnval: program::Data::Number(dec!(0)),
    };
    program.run_loop(&mut Vec::new());
    match program.returnval {
        program::Data::Number(e) => exit(e.round().to_string().parse().unwrap()),
        _ => unimplemented!(),
    }
}
