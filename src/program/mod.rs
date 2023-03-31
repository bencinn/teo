pub mod commands;

pub struct Program {
    pub commands: Vec<commands::Command>,
    pub current_line: usize,
    pub panic: bool,
}

impl Program {
    pub fn run_command(&mut self, writer: &mut impl std::io::Write) {
        if let Some(current_command) = self.commands.get(self.current_line) {
            match current_command.commands_name.as_str() {
                "print" => {
                    for i in &current_command.commands_param {
                        let output = i.get_value_as_str();
                        writeln!(writer, "{}", output).unwrap();
                        println!("{}", output);
                    }
                }
                _ => {
                    writeln!(
                        writer,
                        "Command not exist: {}",
                        current_command.commands_name
                    )
                    .unwrap();
                    println!("Command not exist: {}", current_command.commands_name);
                }
            }
            self.current_line += 1;
        } else {
            self.panic = true;
        }
    }

    pub fn run_loop(self: Program, mut writer: &mut impl std::io::Write) -> Self {
        let mut program = self;
        while !program.panic && program.current_line < program.commands.len() {
            program.run_command(&mut writer);
        }
        program
    }
}

#[cfg(test)]
mod program {
    use super::commands::Command;
    use super::Program;

    #[test]
    fn test_program_panic() {
        let command = Command::new("stuff".to_owned(), "1,23".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let mut program: Program = Program {
            commands: vec_commands,
            current_line: 1,
            panic: false,
        };
        program.run_command(&mut Vec::new());
        assert_eq!(program.panic, true);
        assert_eq!(program.current_line, 1);
    }

    #[test]
    fn test_program_out_of_bounds() {
        let command = Command::new("stuff".to_owned(), "1,23".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let mut program: Program = Program {
            commands: vec_commands,
            current_line: 2,
            panic: false,
        };
        program.run_command(&mut Vec::new());
        assert_eq!(program.panic, true);
        assert_eq!(program.current_line, 2);
    }

    #[test]
    fn test_program_panic_variable() {
        let command = Command::new("stuff".to_owned(), "1,23".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let mut program: Program = Program {
            commands: vec_commands,
            current_line: 2,
            panic: true,
        };
        program.run_command(&mut Vec::new());
        assert_eq!(program.panic, true);
        assert_eq!(program.current_line, 2);
    }
    #[test]
    fn test_program_run_correctly() {
        let command = Command::new("stuff".to_owned(), "1,23".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 0,
            panic: false,
        };
        let mut result = Vec::new();
        program.run_loop(&mut result);
        assert_eq!(result, b"Command not exist: stuff\n");
    }
    #[test]
    fn test_program_print_command() {
        let command = Command::new("print".to_owned(), "\"Hello world!\"".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 0,
            panic: false,
        };
        let mut result = Vec::new();
        let program_result = program.run_loop(&mut result);
        assert_eq!(program_result.current_line, 1);
        assert_eq!(result, b"Hello world!\n");
    }

    #[test]
    fn test_program_multiple_print_commands() {
        let command1 = Command::new("print".to_owned(), "\"Hello\"".to_owned());
        let command2 = Command::new("print".to_owned(), "\"world!\"".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command1);
        vec_commands.push(command2);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 0,
            panic: false,
        };
        let mut result = Vec::new();
        let program_result = program.run_loop(&mut result);
        assert_eq!(program_result.current_line, 2);
        assert_eq!(result, b"Hello\nworld!\n");
    }

    #[test]
    fn test_program_empty_print_command() {
        let command = Command::new("print".to_owned(), "\"\"".to_owned());
        let mut vec_commands: Vec<Command> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 0,
            panic: false,
        };
        let result = program.run_loop(&mut Vec::new());
        assert_eq!(result.current_line, 1);
    }
}
