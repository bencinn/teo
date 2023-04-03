use std::collections::HashMap;

pub struct Param {
    pub param_type: String,
    pub param: String,
}
impl Param {
    pub fn new_param(param_value: String) -> Param {
        let param_type = if param_value.starts_with('"') && param_value.ends_with('"') {
            "String"
        } else {
            match &param_value.parse::<f64>() {
                Ok(_) => "Number",
                Err(_) => "Identifier",
            }
        };

        Param {
            param: param_value,
            param_type: param_type.to_owned(),
        }
    }
    pub fn param_from_vec(param_as_string_vec: Vec<String>) -> Vec<Param> {
        let mut vec_param: Vec<Param> = Vec::new();
        for i in param_as_string_vec {
            vec_param.push(Param::new_param(i));
        }
        vec_param
    }
    pub fn get_value_as_str(&self, program_variable: &HashMap<String, f32>) -> String {
        match self.param_type.as_str() {
            "String" => {
                let mut param_chars = self.param.chars();
                param_chars.next();
                param_chars.next_back();
                String::from(param_chars.as_str())
            }
            "Number" => self.param.parse::<f64>().unwrap().to_string(),
            "Identifier" => program_variable
                .get(self.param.as_str())
                .map(|&value| value.to_string())
                .unwrap_or_else(|| self.param.clone()),
            &_ => {
                panic!("Cannot get value as string for type {}", self.param_type);
            }
        }
    }
    pub fn get_value_as_float(&self, program_variable: &HashMap<String, f32>) -> f32 {
        match self.param_type.as_str() {
            "Number" => self.param.parse::<f64>().unwrap() as f32,
            // TODO: Fix this (This will panic if the variable indexed does not exist.
            "Identifier" => program_variable[self.param.as_str()],
            &_ => {
                panic!("Cannot get value as float for type {}", self.param_type);
            }
        }
    }
    pub fn get_value_as_varname(&self) -> &str {
        match self.param_type.as_str() {
            "Identifier" => self.param.as_str(),
            &_ => {
                panic!("Cannot get varname for {}", self.param_type);
            }
        }
    }
}
