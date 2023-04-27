use std::fs;

use super::parser::Ast;

pub mod param;

pub struct Command {
    pub commands_name: String,
    pub commands_param: Vec<param::Param>,
}

impl Command {
    pub fn new(name: String, param: String) -> Self {
        let mut commands_vec = Vec::new();
        let mut buf: String = String::new();
        let mut in_quotes = false;
        for c in param.chars() {
            if c == '"' {
                in_quotes = !in_quotes;
                buf.push(c);
            } else if c == ',' && !in_quotes {
                commands_vec.push(buf.trim().to_owned());
                buf.clear();
            }
            else {
                buf.push(c);
            }
        }
        commands_vec.push(buf.trim().to_owned());

        Command {
            commands_name: name,
            commands_param: param::Param::param_from_vec(commands_vec),
        }
    }
    pub fn from_text(contents: String) -> Vec<Command> {
        Ast::parse_code(&contents);
        let mut vec_commands: Vec<Command> = Vec::new();
        for i in contents.trim().split(";") {
            if !i.is_empty() {
                let start = i.find("(").unwrap_or(i.len());
                let command_name = i[0..start].trim().to_owned();
                let params = i[start..].trim_matches(|c| c == '(' || c == ')').to_owned();
                let command = Command::new(command_name, params);
                vec_commands.push(command);
            }
        }
        vec_commands
    }
    pub fn read_file(file_name: &str) -> Result<Vec<Command>, std::io::Error> {
        let contents = fs::read_to_string(file_name)?;
        Ok(Command::from_text(contents))
    }
}

#[cfg(test)]
mod test_commands {
    use super::Command;
    use crate::program::commands::param;
    use std::fs;

    #[test]
    fn test_new_command_does_not_crash() {
        let name = String::from("test");
        let param = String::from("1, 2, 3");

        let _command = Command::new(name, param);
        assert!(true, "Commands should not crash");
    }

    #[test]
    fn test_new_command_equals_initialized() {
        let name = String::from("test");
        let param = String::from("1, 2, 3");

        let command = Command::new(name.clone(), param.clone());
        let expected = Command {
            commands_name: name,
            commands_param: param::Param::param_from_vec(vec![
                String::from("1"),
                String::from("2"),
                String::from("3"),
            ]),
        };

        assert_eq!(
            command.commands_name, expected.commands_name,
            "Command name should matched"
        );
        let mut x = 0;
        for i in command.commands_param {
            assert_eq!(i.param, expected.commands_param[x].param);
            x += 1;
        }
    }

    #[test]
    fn test_new_command_trailing_spaces() {
        let name = String::from("test");
        let param = String::from("1, 2,  3");

        let command = Command::new(name, param);
        let expected = vec![String::from("1"), String::from("2"), String::from("3")];

        for (actual_param, expected_param) in command.commands_param.iter().zip(expected.iter()) {
            assert_eq!(
                &actual_param.param, expected_param,
                "{}'s trailing spaces should be removed to {}",
                actual_param.param, expected_param
            );
        }
    }
    #[test]
    fn test_new_command_double_quotes() {
        let name = String::from("test");
        let param = String::from("\"param1, param2\", 3");

        let command = Command::new(name.clone(), param.clone());
        let expected = Command {
            commands_name: name,
            commands_param: param::Param::param_from_vec(vec![
                String::from("\"param1, param2\""),
                String::from("3"),
            ]),
        };

        let mut x = 0;
        for i in command.commands_param {
            assert_eq!(i.param, expected.commands_param[x].param);
            x += 1;
        }
    }
    #[test]
    fn test_read_from_text() {
        let commands_in_text = String::from("test(1, 2);test2(2, 3)");
        let vec_commands = Command::from_text(commands_in_text);
        let expected = vec![
            Command {
                commands_name: "test".to_string(),
                commands_param: param::Param::param_from_vec(vec![
                    "1".to_string(),
                    "2".to_string(),
                ]),
            },
            Command {
                commands_name: "test2".to_string(),
                commands_param: param::Param::param_from_vec(vec![
                    "2".to_string(),
                    "3".to_string(),
                ]),
            },
        ];

        for (actual, expected) in vec_commands.iter().zip(expected.iter()) {
            assert_eq!(
                actual.commands_name, expected.commands_name,
                "Command name should match"
            );
            let mut x = 0;
            actual.commands_param.iter().for_each(|i| {
                assert_eq!(i.param, expected.commands_param[x].param);
                x += 1;
            });
        }
    }
    #[test]
    fn test_read_from_file() {
        fs::write("test.teo", b"test(1, 2);test2(2, 3)").unwrap();
        let vec_return = match Command::read_file("test.teo") {
            Ok(vec) => vec,
            Err(e) => panic!("Failed to read file: {:?}", e),
        };
        let expected = vec![
            Command {
                commands_name: "test".to_string(),
                commands_param: param::Param::param_from_vec(vec![
                    "1".to_string(),
                    "2".to_string(),
                ]),
            },
            Command {
                commands_name: "test2".to_string(),
                commands_param: param::Param::param_from_vec(vec![
                    "2".to_string(),
                    "3".to_string(),
                ]),
            },
        ];

        for (actual, expected) in vec_return.iter().zip(expected.iter()) {
            assert_eq!(
                actual.commands_name, expected.commands_name,
                "Command names should match"
            );
            let mut x = 0;
            actual.commands_param.iter().for_each(|i| {
                assert_eq!(i.param, expected.commands_param[x].param);
                x += 1;
            });
        }
        fs::remove_file("test.teo").unwrap();
    }
    #[test]
    fn test_read_from_nonexistent_file() {
        match Command::read_file("nonexistent_file.teo") {
            Ok(_) => panic!("Expected an error, but read_file succeeded"),
            Err(e) => assert_eq!(
                e.kind(),
                std::io::ErrorKind::NotFound,
                "Expected an error with NotFound kind, but got {:?}",
                e.kind()
            ),
        };
    }
}
