use crate::util::shell::Shell;
use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;

pub mod parser;
pub struct Program {
    pub commands: parser::Ast,
    pub current_line: usize,
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

macro_rules! matchcmd {
    ($id:expr, {$($function:expr => $body:block),+}) => {
        match $id.as_str() {
            $(
                #[cfg(feature = $function)]
                $function => $body,
            )+
            _ => Err(anyhow!("Function isn't enabled")),
        }
    };
}

macro_rules! fep {
    ($program:ident, $args:expr, $parseto:ident, $writer:ident $body:block) => {
        for arg in $args {
            let $parseto = arg.evaluate(&$program, $writer).unwrap();
            $body
        }
    };
}

macro_rules! unrecov_err {
    ($shell:ident, $($errormessage:tt)*) => {
        let mesg = format!($($errormessage)*);
        let _ = $shell.error(format!("Unrecoverable error: {}", mesg));
        panic!("Unrecoverable error!")
    }
}

impl Program {
    pub fn run_loop(
        &mut self,
        mut writer: &mut impl std::io::Write,
        shell: &mut Shell,
    ) -> Result<Data> {
        match &self.commands {
            parser::Ast::Block(commands) => {
                for command in commands {
                    match command.1 {
                        parser::Ast::Set { id, expr } => {
                            let value = expr.evaluate(&self, writer);
                            match id.as_ref() {
                                parser::Ast::ArrayCall { id: array_id, k } => {
                                    let index = k
                                        .evaluate(&self, writer)
                                        .unwrap()
                                        .as_number()
                                        .to_usize()
                                        .unwrap();

                                    let array = self.variable.get_mut(array_id);
                                    if let Some(array) = array {
                                        if let Data::Array(elements) = array {
                                            elements[index] = value.unwrap();
                                        } else {
                                            unrecov_err!(
                                                shell,
                                                "Variable {} is not an array, cannot modify!",
                                                array_id
                                            );
                                        }
                                    } else {
                                        unrecov_err!(
                                            shell,
                                            "Variable (array) not found: {}",
                                            array_id
                                        );
                                    }
                                }
                                _ => {
                                    self.variable.insert(id.to_string(), value.unwrap());
                                }
                            };
                        }
                        parser::Ast::If { condition, block } => {
                            let conditionresult = condition.evaluate(&self, writer);
                            match conditionresult.unwrap() {
                                Data::Bool(e) => {
                                    if e {
                                        let mut program = Program {
                                            commands: *block.clone(),
                                            current_line: 0,
                                            variable: self.variable.clone(),
                                            function: self.function.clone(),
                                            std_commands: self.std_commands.clone(),
                                        };
                                        if let Ok(_) = program.run_loop(writer, shell) {
                                            self.variable = program.variable;
                                        } else {
                                            panic!("Code block within If-else panicked!");
                                        }
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
                                        });
                                            Ok(Data::Number(dec!(0)))
                                    },
                                    "return" => {
                                        if let Some(arg) = args.first() {
                                            let value = arg.evaluate(&self, writer).unwrap();
                                            Ok(value)
                                        } else {
                                            Err(anyhow!("Need to return only one value!"))
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
                                            let value = arg.evaluate(&self, writer).unwrap();
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
                                            variable: local_variables,
                                            function: self.function.clone(),
                                            std_commands: self.std_commands.clone(),
                                        };
                                        if let Ok(returncode) = program.run_loop(writer, shell) {
                                        } else {
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
        Ok(Data::Number(dec!(0)))
    }
}

trait Evaluate {
    fn evaluate(&self, program: &Program, writer: &mut impl std::io::Write) -> Result<Data>;
}

impl Evaluate for parser::Ast {
    fn evaluate(&self, program: &Program, mut writer: &mut impl std::io::Write) -> Result<Data> {
        let variables = &program.variable;
        match self {
            parser::Ast::Int(i) => Ok(Data::Number(*i)),
            parser::Ast::Bool(b) => Ok(Data::Bool(*b)),
            parser::Ast::Identifier(id) => match variables.get(id) {
                Some(value) => Ok(value.clone()),
                None => Err(anyhow!("Error: variable not found: {}", id)),
            },
            parser::Ast::BinaryOp { op, left, right } => {
                let left_value = left.evaluate(program, writer).unwrap();
                let right_value = right.evaluate(program, writer).unwrap();
                let f1 = left_value.as_number();
                let f2 = right_value.as_number();
                match op.as_str() {
                    "+" => Ok(Data::Number(f1 + f2)),
                    "-" => Ok(Data::Number(f1 - f2)),
                    "*" => Ok(Data::Number(f1 * f2)),
                    "/" => Ok(Data::Number(f1 / f2)),
                    "==" => Ok(Data::Bool(f1 == f2)),
                    "!=" => Ok(Data::Bool(f1 != f2)),
                    "<" => Ok(Data::Bool(f1 < f2)),
                    ">" => Ok(Data::Bool(f1 > f2)),
                    "<=" => Ok(Data::Bool(f1 <= f2)),
                    ">=" => Ok(Data::Bool(f1 >= f2)),
                    _ => panic!("{} is not a valid binary operator", op),
                }
            }
            parser::Ast::String(i) => Ok(Data::String(i.clone())),
            parser::Ast::Array(elements) => {
                let mut array_data = Vec::new();
                for element in elements {
                    let element_data = element.evaluate(program, writer).unwrap();
                    array_data.push(element_data);
                }
                Ok(Data::Array(array_data))
            }
            parser::Ast::ArrayCall { id, k } => {
                if let Some(array) = variables.get(id) {
                    if let Data::Array(elements) = array {
                        let index = k
                            .evaluate(program, writer)
                            .unwrap()
                            .as_number()
                            .to_usize()
                            .unwrap();
                        if index >= elements.len() {
                            panic!("Error: array index out of bounds");
                        }
                        Ok(elements[index].clone())
                    } else {
                        Err(anyhow!(format!(
                            "Variable {} is not an array, cannot modify!",
                            id
                        )))
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
                                Ok(Data::Number(dec!(0)))
                        },
                        "return" => {
                            if let Some(arg) = args.first() {
                                let value = arg.evaluate(&program, writer).unwrap();
                                Ok(value)
                            } else {
                                Err(anyhow!("Need to return only one value!"))
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
                                let value = arg.evaluate(program, writer).unwrap();
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
                                variable: local_variables,
                                function: program.function.clone(),
                                std_commands: program.std_commands.clone(),
                            };
                            if let Ok(returncode) = program.run_loop(writer, &mut Shell::new()) {
                                Ok(returncode)
                            } else {
                                panic!("Function `{}` panicked!", id);
                            }
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
