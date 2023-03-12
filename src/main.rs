use std::process;

use clap::Parser;

use program::commands::Command;
use program::Program;

mod program;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Read from <FILE_NAME>
    #[arg(short, long)]
    file_name: String,
}

fn main() {
    let args = Args::parse();
    let vec_commands = match Command::read_file(&args.file_name) {
        Ok(vec) => vec,
        Err(_e) => {
            eprintln!("Failed to read file");
            process::exit(1);
        }
    };
    let program: Program = Program {
        commands: vec_commands,
        current_line: 0,
        panic: false,
    };
    program.run_loop(&mut Vec::new());
}
