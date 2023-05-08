#![feature(test)]

use clap::Parser;
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
    // Only parse the code and exit
    #[arg(long, default_value_t = false)]
    only_parse: bool,
}

fn main() {
    let args = Args::parse();
    let vec_ast = match program::parser::Ast::parse_code(
        fs::read_to_string(args.file_name).unwrap().as_str(),
    ) {
        Ok(ast) => ast,
        Err(_e) => {
            eprintln!("Failed to read file");
            process::exit(1);
        }
    };
    if args.only_parse {
        exit(0);
    };
    let mut program: Program = Program {
        commands: vec_ast,
        current_line: 0,
        panic: false,
        variable: HashMap::new(),
        function: HashMap::new(),
        std_commands: Vec::from([
            "return".to_owned(),
            "print".to_owned(),
            "printstr".to_owned(),
        ]),
    };
    program.run_loop(&mut Vec::new());
}
