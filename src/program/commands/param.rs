use eval::eval;

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
                Err(_) => match eval(&param_value) {
                    Ok(_) => "Eval",
                    Err(_) => panic!("Unable to determine type of parameter: {}", param_value),
                },
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
    pub fn get_value_as_str(&self) -> String {
        match self.param_type.as_str() {
            "String" => {
                let mut param_chars = self.param.chars();
                param_chars.next();
                param_chars.next_back();
                String::from(param_chars.as_str())
            }
            "Number" => self.param.parse::<f64>().unwrap().to_string(),
            "Eval" => match eval(self.param.as_str()) {
                Ok(result) => result.to_string(),
                Err(_) => panic!("Cannot evaluate result"),
            },
            &_ => {
                panic!("Cannot get value as string for type {}", self.param_type);
            }
        }
    }
}
