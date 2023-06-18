use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;
pub mod parser;
pub struct Program {
    pub commands: parser::Ast,
    pub current_line: usize,
    pub panic: bool,
    pub variable: HashMap<String, Data>,
    pub function: HashMap<String, parser::Ast>,
    pub std_commands: Vec<String>,
    pub returnval: Data,
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

macro_rules! matchcmd {
    ($id:expr, {$($function:expr => $body:block),+}) => {
        match $id.as_str() {
            $(
                #[cfg(feature = $function)]
                $function => $body,
            )+
            _ => panic!("Function isn't enabled"),
        }
    };
}

macro_rules! fep {
    ($program:ident, $args:expr, $parseto:ident, $writer:ident $body:block) => {
        for arg in $args {
            let $parseto = arg.evaluate(&$program, $writer);
            $body
        }
    };
}

impl Program {
    pub fn run_loop(&mut self, mut writer: &mut impl std::io::Write) {
        match &self.commands {
            parser::Ast::Block(commands) => {
                for command in commands {
                    match command.1 {
                        parser::Ast::Set { id, expr } => {
                            let value = expr.evaluate(&self, writer);
                            match id.as_ref() {
                                parser::Ast::ArrayCall { id: array_id, k } => {
                                    let index =
                                        k.evaluate(&self, writer).as_number().to_usize().unwrap();

                                    let array = self.variable.get_mut(array_id);
                                    if let Some(array) = array {
                                        if let Data::Array(elements) = array {
                                            elements[index] = value;
                                        } else {
                                            panic!(
                                                "Variable {} is not an array, cannot modify!",
                                                id
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
                            let conditionresult = condition.evaluate(&self, writer);
                            match conditionresult {
                                Data::Bool(e) => {
                                    if e {
                                        let mut program = Program {
                                            commands: *block.clone(),
                                            current_line: 0,
                                            panic: false,
                                            variable: self.variable.clone(),
                                            function: self.function.clone(),
                                            std_commands: self.std_commands.clone(),
                                            returnval: Data::Number(dec!(0)),
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
                                matchcmd!(id, {
                                    "print" => {
                                        fep!(self, args, value, writer {
                                            println!("{}", value.as_string());
                                            write!(&mut writer, "{}", value.as_string()).unwrap();
                                        })
                                    },
                                    "return" => {
                                        if let Some(arg) = args.first() {
                                            let value = arg.evaluate(&self, writer);
                                            self.returnval = value
                                        } else {
                                            panic!("Need exit code for the return function!");
                                        }
                                    }
                                }
                                );
                            } else if let Some(func) = self.function.get(id) {
                                match func {
                                    parser::Ast::FunctionDefinition { params, body, .. } => {
                                        if params.len() < args.len() {
                                            panic!("Too many argument!");
                                        }
                                        if params.len() > args.len() {
                                            panic!("Not enough argument!");
                                        }
                                        let mut local_variables = HashMap::new();
                                        for (i, arg) in args.iter().enumerate() {
                                            let (name, dtype) = &params[i];
                                            let value = arg.evaluate(&self, writer);
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
                                            commands: *body.clone(),
                                            current_line: 0,
                                            panic: false,
                                            variable: local_variables,
                                            function: self.function.clone(),
                                            std_commands: self.std_commands.clone(),
                                            returnval: Data::Number(dec!(0)),
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
            _ => unimplemented!(),
        }
    }
}

trait Evaluate {
    fn evaluate(&self, program: &Program, writer: &mut impl std::io::Write) -> Data;
}

impl Evaluate for parser::Ast {
    fn evaluate(&self, program: &Program, mut writer: &mut impl std::io::Write) -> Data {
        let variables = &program.variable;
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
                let left_value = left.evaluate(program, writer);
                let right_value = right.evaluate(program, writer);
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
                    let element_data = element.evaluate(program, writer);
                    array_data.push(element_data);
                }
                Data::Array(array_data)
            }
            parser::Ast::ArrayCall { id, k } => {
                if let Some(array) = variables.get(id) {
                    if let Data::Array(elements) = array {
                        let index = k.evaluate(program, writer).as_number().to_usize().unwrap();
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

            parser::Ast::FunctionCall { id, args } => {
                let std_functions = program.std_commands.clone();
                if std_functions.contains(id) {
                    matchcmd!(id, {
                        "print" => {
                            fep!(program, args, value, writer {
                                println!("{}", value.as_string());
                                write!(&mut writer, "{}", value.as_string()).unwrap();
                            });
                            Data::Number(dec!(0))
                        },
                        "return" => {
                            if let Some(arg) = args.first() {
                                let value = arg.evaluate(program, writer);
                                value
                            } else {
                                panic!("Need exit code for the return function!");
                            }
                        }
                    }
                    )
                } else if let Some(func) = program.function.get(id) {
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
                                let value = arg.evaluate(program, writer);
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
                                commands: *body.clone(),
                                current_line: 0,
                                panic: false,
                                variable: local_variables,
                                function: program.function.clone(),
                                std_commands: program.std_commands.clone(),
                                returnval: Data::Number(dec!(0)),
                            };
                            program.run_loop(writer);
                            if program.panic {
                                panic!("Function `{}` panicked!", id);
                            }
                            program.returnval
                        }
                        _ => panic!("`{}` is not a function!", id),
                    }
                } else {
                    panic!("Function `{}` is not defined!", id);
                }
            }
            _ => panic!("Invalid AST node"),
        }
    }
}
