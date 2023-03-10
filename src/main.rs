use clap::Parser;
use std::fs;

use program::commands::Commands;
use program::Program;

mod program;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let vec_commands: Vec<Commands> = read_file(&args.file);
    let program: Program = Program {
        commands: vec_commands,
        current_line: 0,
        panic: false,
    };
    program.run_loop();
}
fn read_file(file_name: &str) -> Vec<Commands> {
    let mut vec_commands: Vec<Commands> = Vec::new();
    let contents = fs::read_to_string(file_name).expect("Unable to read the file");
    for i in contents.split("\n") {
        let start = i.find("(").unwrap_or(i.len());
        let command_name = i[0..start].trim().to_owned();
        let params = i[start..].trim_matches(|c| c == '(' || c == ')').to_owned();
        if command_name.is_empty() {
            panic!("Invalid command string: {}", i);
        }
        if params.is_empty() {
            panic!("Invalid parameter string: {}", i);
        }
        let start = i.find("(").unwrap_or(i.len());
        let command_name = i[0..start].trim().to_owned();
        let params = i[start..].trim_matches(|c| c == '(' || c == ')').to_owned();
        let command = Commands::new(command_name, params);
        vec_commands.push(command);
    }
    vec_commands
}

#[cfg(test)]
mod read_file {
    #[test]
    #[should_panic(expected = "Unable to read the file")]
    fn test_nonexistent_file() {
        use super::read_file;
        read_file("nonexistent_file.txt");
    }
}
