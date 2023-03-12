pub struct Param {
    pub param_type: String,
    pub param: String,
}
impl Param {
    pub fn new_param(param_value: String) -> Param {
        Param {
            param: param_value,
            param_type: "String".to_owned(),
        }
    }
    pub fn param_from_vec(param_as_string_vec: Vec<String>) -> Vec<Param> {
        let mut vec_param: Vec<Param> = Vec::new();
        for i in param_as_string_vec {
            vec_param.push(Param::new_param(i));
        }
        vec_param
    }
}
