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
}

#[cfg(test)]
mod test_commands {
    use super::Commands;

    #[test]
    fn test_commands_does_not_crash() {
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
    fn test_trailing_spaces() {
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
}
