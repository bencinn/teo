use program::commands::Commands;
use program::Program;

mod program;
fn main() {
    let command = Commands::new("stuff".to_owned(), "aa,bb".to_owned());
    let mut vec_commands: Vec<Commands> = Vec::new();
    vec_commands.push(command);
    let program: Program = Program {
        commands: vec_commands,
        current_line: 0,
        panic: false,
    };
    program.run_loop();
}
