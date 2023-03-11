use std::fs;

pub struct Commands {
    pub commands_name: String,
    pub commands_param: Vec<String>,
}

impl Commands {
    pub fn new(name: String, param: String) -> Self {
        let mut commands_vec = Vec::new();
        let mut buf: String = String::new();
        for c in param.chars() {
            if c == ',' {
                commands_vec.push(buf.trim().to_owned());
                buf.clear();
            } else {
                buf.push(c);
            }
        }
        commands_vec.push(buf.trim().to_owned());

        Commands {
            commands_name: name,
            commands_param: commands_vec,
        }
    }
    pub fn from_text(contents: String) -> Vec<Commands> {
        let mut vec_commands: Vec<Commands> = Vec::new();
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
    pub fn read_file(file_name: &str) -> Vec<Commands> {
        let contents = fs::read_to_string(file_name).expect("Unable to read the file");
        Commands::from_text(contents)
    }
}

#[cfg(test)]
mod test_commands {
    use super::Commands;
    use std::fs;

    #[test]
    fn test_new_command_does_not_crash() {
        let name = String::from("test");
        let param = String::from("param1, param2, param3");

        let _command = Commands::new(name, param);
        assert!(true, "Commands should not crash");
    }

    #[test]
    fn test_new_command_equals_initialized() {
        let name = String::from("test");
        let param = String::from("param1, param2, param3");

        let command = Commands::new(name.clone(), param.clone());
        let expected = Commands {
            commands_name: name,
            commands_param: vec![
                String::from("param1"),
                String::from("param2"),
                String::from("param3"),
            ],
        };

        assert_eq!(
            command.commands_name, expected.commands_name,
            "Command name should matched"
        );
        assert_eq!(
            command.commands_param, expected.commands_param,
            "Command params should matched"
        );
    }

    #[test]
    fn test_new_command_trailing_spaces() {
        let name = String::from("test");
        let param = String::from("param1, param2 ,param3 ");

        let command = Commands::new(name, param);
        let expected = vec![
            String::from("param1"),
            String::from("param2"),
            String::from("param3"),
        ];

        for (actual_param, expected_param) in command.commands_param.iter().zip(expected.iter()) {
            assert_eq!(
                actual_param, expected_param,
                "{}'s trailing spaces should be removed to {}",
                actual_param, expected_param
            );
        }
    }
    #[test]
    fn test_read_from_text() {
        let commands_in_text = String::from("test(aa,bb,cc)\ntest2(a,b,cc)");
        let vec_commands = Commands::from_text(commands_in_text);
        let expected = vec![
            Commands {
                commands_name: "test".to_string(),
                commands_param: vec!["aa".to_string(), "bb".to_string(), "cc".to_string()],
            },
            Commands {
                commands_name: "test2".to_string(),
                commands_param: vec!["a".to_string(), "b".to_string(), "cc".to_string()],
            },
        ];

        for (actual, expected) in vec_commands.iter().zip(expected.iter()) {
            assert_eq!(
                actual.commands_name, expected.commands_name,
                "Command name should match"
            );
            assert_eq!(
                actual.commands_param, expected.commands_param,
                "Command params should match"
            );
        }
    }
    #[test]
    fn test_read_from_file() {
        fs::write("test.teo", b"test(aa,bb,cc)\ntest2(a,b,cc)").unwrap();
        let vec_return = Commands::read_file("test.teo");
        let expected = vec![
            Commands {
                commands_name: String::from("test"),
                commands_param: vec![String::from("aa"), String::from("bb"), String::from("cc")],
            },
            Commands {
                commands_name: String::from("test2"),
                commands_param: vec![String::from("a"), String::from("b"), String::from("cc")],
            },
        ];

        for (actual, expected) in vec_return.iter().zip(expected.iter()) {
            assert_eq!(
                actual.commands_name, expected.commands_name,
                "Command names should match"
            );
            assert_eq!(
                actual.commands_param, expected.commands_param,
                "Command parameters should match"
            );
        }
        fs::remove_file("test.teo").unwrap();
    }
}
