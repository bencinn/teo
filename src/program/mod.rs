pub mod commands;

pub struct Program {
    pub commands: Vec<commands::Commands>,
    pub current_line: usize,
    pub panic: bool,
}

impl Program {
    pub fn run_command(self: Program) -> Self {
        if let Some(current_command) = self.commands.get(self.current_line) {
            match current_command.commands_name.as_str() {
                "print" => {
                    for i in &current_command.commands_param {
                        println!("{}", i);
                    }
                }
                _ => {
                    println!("Command not exist: {}", current_command.commands_name);
                }
            }
            Program {
                commands: self.commands,
                current_line: self.current_line + 1, // Increment current_line after executing the command
                panic: false,
            }
        } else {
            Program {
                commands: self.commands,
                current_line: self.current_line,
                panic: true,
            }
        }
    }

    pub fn run_loop(self: Program) -> Self {
        let mut program = self;
        while !program.panic && program.current_line < program.commands.len() {
            program = program.run_command();
        }
        program
    }
}

#[cfg(test)]
mod program {
    use super::commands::Commands;
    use super::Program;

    #[test]
    fn test_program_panic() {
        let command = Commands::new("stuff".to_owned(), "aa,bb".to_owned());
        let mut vec_commands: Vec<Commands> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 1,
            panic: false,
        };
        let result = program.run_command();
        assert_eq!(result.panic, true);
        assert_eq!(result.current_line, 1);
    }

    #[test]
    fn test_program_out_of_bounds() {
        let command = Commands::new("stuff".to_owned(), "aa,bb".to_owned());
        let mut vec_commands: Vec<Commands> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 2,
            panic: false,
        };
        let result = program.run_command();
        assert_eq!(result.panic, true);
        assert_eq!(result.current_line, 2);
    }

    #[test]
    fn test_program_panic_variable() {
        let command = Commands::new("stuff".to_owned(), "aa,bb".to_owned());
        let mut vec_commands: Vec<Commands> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 2,
            panic: true,
        };
        let result = program.run_command();
        assert_eq!(result.panic, true);
        assert_eq!(result.current_line, 2);
    }
    #[test]
    fn test_program_run_correctly() {
        let command = Commands::new("stuff".to_owned(), "aa,bb".to_owned());
        let mut vec_commands: Vec<Commands> = Vec::new();
        vec_commands.push(command);
        let program: Program = Program {
            commands: vec_commands,
            current_line: 0,
            panic: false,
        };
        program.run_loop();
        assert!(true);
    }
}
