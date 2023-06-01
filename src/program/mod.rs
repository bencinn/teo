use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
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

#[derive(Clone, PartialEq, Debug)]
pub enum Data {
    String(String),
    Number(Decimal),
    Array(Vec<Data>),
    Bool(bool),
}

impl Data {
    fn as_number(&self) -> Decimal {
        match self {
            Data::Number(i) => *i,
            Data::Bool(b) => {
                if *b {
                    dec!(1)
                } else {
                    dec!(0)
                }
            }
            _ => panic!("Data is not convertable"),
        }
    }
    fn as_string(&self) -> String {
        match self {
            Data::Number(i) => i.normalize().to_string(),
            Data::String(i) => i.clone(),
            Data::Bool(b) => b.to_string(),
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
                    match id.as_ref() {
                        parser::Ast::ArrayCall { id: array_id, k } => {
                            let index = k.evaluate(&self.variable).as_number().to_usize().unwrap();

                            let array = self.variable.get_mut(array_id);
                            if let Some(array) = array {
                                if let Data::Array(elements) = array {
                                    elements[index] = value;
                                } else {
                                    panic!(
                                        "Variable {} is not an array, cannot modify!",
                                        id.to_string()
                                    );
                                }
                            } else {
                                panic!("Variable (array) not found: {}", array_id);
                            }
                        }
                        _ => {
                            self.variable.insert(id.to_string(), value);
                        }
                    };
                }
                parser::Ast::If { condition, block } => {
                    let conditionresult = condition.evaluate(&self.variable);
                    match conditionresult {
                        Data::Bool(e) => {
                            if e {
                                let mut program = Program {
                                    commands: block.clone(),
                                    current_line: 0,
                                    panic: false,
                                    variable: self.variable.clone(),
                                    function: self.function.clone(),
                                    std_commands: self.std_commands.clone(),
                                };
                                program.run_loop(writer);
                                if program.panic {
                                    panic!("Code block within If-else panicked!");
                                }
                                self.variable = program.variable;
                            }
                        }
                        _ => unimplemented!(),
                    };
                }
                parser::Ast::FunctionDefinition { id, params, body } => {
                    if self.function.contains_key(id) | self.std_commands.contains(id) {
                        panic!("Function `{}` already exist!", id);
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
                    if std_functions.contains(id) {
                        match id.as_str() {
                            #[cfg(feature = "print")]
                            "print" => {
                                for arg in args {
                                    let value = arg.evaluate(&self.variable);
                                    println!("{}", value.as_string());
                                    write!(&mut writer, "{}", value.as_string()).unwrap();
                                }
                            }
                            #[cfg(feature = "return")]
                            "return" => {
                                if let Some(arg) = args.first() {
                                    let value = arg.evaluate(&self.variable);
                                    exit(value.as_number().to_i32().unwrap());
                                } else {
                                    panic!("Need exit code for the return function!");
                                }
                            }
                            _ => panic!("Function isn't enabled"),
                        }
                    } else if let Some(func) = self.function.get(id) {
                        match func {
                            parser::Ast::FunctionDefinition { params, body, .. } => {
                                if params.len() < args.len() {
                                    panic!("Too many argument!",);
                                }
                                if params.len() > args.len() {
                                    panic!("Not enough argument!",);
                                }
                                let mut local_variables = HashMap::new();
                                for (i, arg) in args.iter().enumerate() {
                                    let (name, dtype) = &params[i];
                                    let value = arg.evaluate(&self.variable);
                                    match dtype.as_str() {
                                        "Integer" => {
                                            if let Data::Number(_) = value {
                                            } else {
                                                panic!("Wrong type for function: expected {}!", dtype);
                                            }
                                        }
                                        "String" => {
                                            if let Data::String(_) = value {
                                            } else {
                                                panic!("Wrong type for function: expected {}!", dtype);
                                            }
                                        }
                                        "Array" => {
                                            if let Data::Array(_) = value {
                                            } else {
                                                panic!("Wrong type for function: expected {}!", dtype);
                                            }
                                        }
                                        "Bool" => {
                                            if let Data::Bool(_) = value {
                                            } else {
                                                panic!("Wrong type for function: expected {}!", dtype);
                                            }
                                        }
                                        _ => panic!("Type does not exist: {}! Only types exist are Integer, String, Bool and Array", dtype),
                                    }
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
                                    panic!("Function `{}` panicked!", id);
                                }
                            }
                            _ => panic!("`{}` is not a function!", id),
                        }
                    } else {
                        panic!("Function `{}` is not defined!", id);
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
            parser::Ast::Int(i) => Data::Number(*i),
            parser::Ast::Bool(b) => Data::Bool(*b),
            parser::Ast::Identifier(id) => match variables.get(id) {
                Some(value) => value.clone(),
                None => {
                    panic!("Error: variable not found: {}", id)
                }
            },
            parser::Ast::BinaryOp { op, left, right } => {
                let left_value = left.evaluate(variables);
                let right_value = right.evaluate(variables);
                let f1 = left_value.as_number();
                let f2 = right_value.as_number();
                match op.as_str() {
                    "+" => Data::Number(f1 + f2),
                    "-" => Data::Number(f1 - f2),
                    "*" => Data::Number(f1 * f2),
                    "/" => Data::Number(f1 / f2),
                    "==" => Data::Bool(f1 == f2),
                    "!=" => Data::Bool(f1 != f2),
                    "<" => Data::Bool(f1 < f2),
                    ">" => Data::Bool(f1 > f2),
                    "<=" => Data::Bool(f1 <= f2),
                    ">=" => Data::Bool(f1 >= f2),
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
            parser::Ast::ArrayCall { id, k } => {
                if let Some(array) = variables.get(id) {
                    if let Data::Array(elements) = array {
                        let index = k.evaluate(variables).as_number().to_usize().unwrap();
                        if index >= elements.len() {
                            panic!("Error: array index out of bounds");
                        }
                        elements[index].clone()
                    } else {
                        panic!("Error: variable {} is not an array", id);
                    }
                } else {
                    panic!("Error: array variable not found: {}", id);
                }
            }
            _ => panic!("Invalid AST node"),
        }
    }
}

extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_evaluate_int() {
        let ast = parser::Ast::Int(Decimal::from(42));
        let variables = HashMap::new();
        let result = ast.evaluate(&variables);
        assert_eq!(result, Data::Number(Decimal::from(42)));
    }

    #[test]
    fn test_evaluate_float() {
        let ast = parser::Ast::Int(Decimal::from_f64(35.8 as f64).unwrap());
        let variables = HashMap::new();
        let result = ast.evaluate(&variables);
        assert_eq!(
            result,
            Data::Number(Decimal::from_f64(35.8 as f64).unwrap())
        );
    }

    #[test]
    fn test_evaluate_identifier() {
        let ast = parser::Ast::Identifier("x".to_string());
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), Data::Number(Decimal::from(42)));
        let result = ast.evaluate(&variables);
        assert_eq!(result, Data::Number(Decimal::from(42)));
    }

    #[test]
    fn test_evaluate_binary_op() {
        let left = parser::Ast::Int(Decimal::from(2));
        let right = parser::Ast::Int(Decimal::from(3));
        let ast = parser::Ast::BinaryOp {
            op: "+".to_string(),
            left: Box::new(left),
            right: Box::new(right),
        };
        let variables = HashMap::new();
        let result = ast.evaluate(&variables);
        assert_eq!(result, Data::Number(Decimal::from(5)));
    }

    #[test]
    fn test_evaluate_string() {
        let ast = parser::Ast::String("hello".to_string());
        let variables = HashMap::new();
        let result = ast.evaluate(&variables);
        assert_eq!(result, Data::String("hello".to_string()));
    }

    #[test]
    fn test_evaluate_array() {
        let elements = vec![
            parser::Ast::Int(dec!(1.0)),
            parser::Ast::Int(dec!(2.0)),
            parser::Ast::Int(dec!(3.0)),
        ];
        let ast = parser::Ast::Array(elements);
        let variables = HashMap::new();
        let result = ast.evaluate(&variables);
        assert_eq!(
            result,
            Data::Array(vec![
                Data::Number(dec!(1.0)),
                Data::Number(dec!(2.0)),
                Data::Number(dec!(3.0))
            ])
        );
    }

    use crate::program::Data;

    #[test]
    fn data_as_number() {
        let data = Data::Number(dec!(1.0));
        assert_eq!(data.as_number(), dec!(1.0));
    }

    #[test]
    fn data_as_string() {
        let data = Data::Number(dec!(1.0));
        assert_eq!(data.as_string(), "1".to_string());
    }

    #[test]
    fn bool_as_string() {
        let data = Data::Bool(true);
        assert_eq!(data.as_string(), "true");
    }

    #[test]
    fn bool_as_number() {
        let data = Data::Bool(true);
        assert_eq!(data.as_number(), dec!(1));
    }

    use rand::{thread_rng, Rng};

    #[bench]
    fn bench_evaluate_int(b: &mut test::Bencher) {
        let mut rng = thread_rng();
        let value = rng.gen_range(10000..50000);
        let ast = parser::Ast::Int(Decimal::from(value));
        let variables = HashMap::new();
        b.iter(|| test::black_box(ast.evaluate(&variables)));
    }

    #[bench]
    fn bench_evaluate_binary_op(b: &mut test::Bencher) {
        let mut rng = thread_rng();
        let left_value = rng.gen_range(10000..50000);
        let right_value = rng.gen_range(10000..50000);
        let left = parser::Ast::Int(Decimal::from(left_value));
        let right = parser::Ast::Int(Decimal::from(right_value));
        let asts = vec![
            parser::Ast::BinaryOp {
                op: "+".to_string(),
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            },
            parser::Ast::BinaryOp {
                op: "-".to_string(),
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            },
            parser::Ast::BinaryOp {
                op: "*".to_string(),
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            },
            parser::Ast::BinaryOp {
                op: "/".to_string(),
                left: Box::new(left),
                right: Box::new(right),
            },
        ];
        let variables = HashMap::new();
        b.iter(|| {
            for ast in &asts {
                test::black_box(ast.evaluate(&variables));
            }
        });
    }
    #[cfg(feature = "print")]
    #[bench]
    fn bench_run_print(b: &mut test::Bencher) {
        let code = "print(1)";
        let ast = parser::Ast::parse_code(code).unwrap();
        let mut program = Program {
            commands: Vec::from(ast),
            current_line: 0,
            panic: false,
            variable: HashMap::new(),
            function: HashMap::new(),
            std_commands: vec!["print".to_owned()],
        };
        b.iter(|| {
            program.run_loop(&mut Vec::new());
        })
    }
}
