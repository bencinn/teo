use std::{collections::HashMap, process::exit};
pub mod parser;
pub struct Program {
    pub commands: Vec<parser::Ast>,
    pub current_line: usize,
    pub panic: bool,
    pub variable: HashMap<String, Data>,
    pub function: HashMap<String, parser::Ast>,
    pub std_commands: Vec<String>,
}

#[derive(Clone)]
pub enum Data {
    String(String),
    Int(f32),
    Array(Vec<Data>),
}

impl Data {
    fn as_float(&self) -> f32 {
        match self {
            Data::Int(i) => *i,
            _ => panic!("Data is not convertable"),
        }
    }
    fn as_string(&self) -> String {
        match self {
            Data::Int(i) => i.to_string(),
            Data::String(i) => i.clone(),
            _ => panic!("Data is not convertable"),
        }
    }
}

impl Program {
    pub fn run_loop(&mut self, mut writer: &mut impl std::io::Write) {
        for command in &self.commands {
            match command {
                parser::Ast::Set { id, expr } => {
                    let value = expr.evaluate(&self.variable);
                    self.variable.insert(id.clone(), value);
                }
                parser::Ast::FunctionDefinition { id, params, body } => {
                    if self.function.contains_key(id) | self.std_commands.contains(id) {
                        panic!("Function `{}` already exist", id);
                    }
                    self.function.insert(
                        id.clone(),
                        parser::Ast::FunctionDefinition {
                            id: id.clone(),
                            params: params.clone(),
                            body: body.clone(),
                        },
                    );
                }
                parser::Ast::FunctionCall { id, args } => {
                    let std_functions = self.std_commands.clone();
                    if std_functions.contains(&id) {
                        match id.as_str() {
                            "print" => {
                                for arg in args {
                                    let value = arg.evaluate(&self.variable);
                                    println!("{}", value.as_string());
                                    write!(&mut writer, "{}", value.as_string()).unwrap();
                                }
                            }
                            "return" => {
                                if let Some(arg) = args.first() {
                                    let value = arg.evaluate(&self.variable);
                                    exit(value.as_float() as i32);
                                } else {
                                    panic!("Need exit code");
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else if let Some(func) = self.function.get(id) {
                        match func {
                            parser::Ast::FunctionDefinition { params, body, .. } => {
                                if params.len() != args.len() {
                                    panic!("Not enough argument",);
                                }
                                let mut local_variables = HashMap::new();
                                for (i, arg) in args.iter().enumerate() {
                                    let (name, _) = &params[i];
                                    let value = arg.evaluate(&self.variable);
                                    local_variables.insert(name.clone(), value);
                                }
                                let mut program = Program {
                                    commands: body.clone(),
                                    current_line: 0,
                                    panic: false,
                                    variable: local_variables,
                                    function: self.function.clone(),
                                    std_commands: self.std_commands.clone(),
                                };
                                program.run_loop(writer);
                                if program.panic {
                                    panic!("Function `{}` panicked", id);
                                }
                            }
                            _ => panic!("`{}` is not a function", id),
                        }
                    } else {
                        panic!("Function `{}` not defined", id);
                    }
                }
                _ => {}
            }
        }
    }
}

trait Evaluate {
    fn evaluate(&self, variables: &HashMap<String, Data>) -> Data;
}

impl Evaluate for parser::Ast {
    fn evaluate(&self, variables: &HashMap<String, Data>) -> Data {
        match self {
            parser::Ast::Int(i) => Data::Int(*i as f32),
            parser::Ast::Identifier(id) => match variables.get(id) {
                Some(value) => value.clone(),
                None => {
                    panic!("Error: variable not found: {}", id)
                }
            },
            parser::Ast::BinaryOp { op, left, right } => {
                let left_value = left.evaluate(variables);
                let right_value = right.evaluate(variables);
                let f1 = left_value.as_float();
                let f2 = right_value.as_float();
                match op.as_str() {
                    "+" => Data::Int(f1 + f2),
                    "-" => Data::Int(f1 - f2),
                    "*" => Data::Int(f1 * f2),
                    "/" => Data::Int(f1 / f2),
                    _ => panic!("{} is not a valid binary operator", op),
                }
            }
            parser::Ast::String(i) => Data::String(i.clone()),
            parser::Ast::Array(elements) => {
                let mut array_data = Vec::new();
                for element in elements {
                    let element_data = element.evaluate(variables);
                    array_data.push(element_data);
                }
                Data::Array(array_data)
            }
            _ => panic!("Invalid AST node"),
        }
    }
}

#[cfg(test)]
mod program {}
