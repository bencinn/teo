use crate::util::shell::Shell;
use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{collections::HashMap, iter::Product};

pub enum ReturnType {
    Ok(Data),
    None,
}

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
    ($id:expr, $errmessage:expr, {$($function:expr => $body:block),+}) => {
        match $id.as_str() {
            $(
                #[cfg(feature = $function)]
                $function => $body,
            )+
            _ => Err(anyhow!($errmessage)),
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
    ) -> Result<ReturnType> {
        match &self.commands {
            parser::Ast::Block(commands) => {
                for command in commands {
                    match command {
                        parser::Ast::Set { id, expr } => {
                            let value = expr.evaluate(&self, writer);
                            match id.as_ref() {
                                parser::Ast::ArrayAccess {
                                    expr: array_id,
                                    whereto: k,
                                } => {
                                    if let parser::Ast::Identifier(id) =
                                        &*std::rc::Rc::clone(&array_id)
                                    {
                                        if let Data::Array(arr) = &self.variable[id] {
                                            let mut a = arr.clone();
                                            a[k.evaluate(&self, writer)
                                                .unwrap()
                                                .as_number()
                                                .to_usize()
                                                .unwrap()] = value.unwrap();
                                            self.variable.insert(id.to_string(), Data::Array(a));
                                        }
                                    } else {
                                        panic!("{:?} {:?}", array_id, k)
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
                                        if let Ok(returnval) = program.run_loop(writer, shell) {
                                            self.variable = program.variable;
                                            match returnval {
                                                ReturnType::Ok(x) => return Ok(ReturnType::Ok(x)),
                                                ReturnType::None => {}
                                            }
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
                                matchcmd!(id, "Function isn't enabled.", {
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
                                            return Ok(ReturnType::Ok(value))
                                        } else {
                                            Err(anyhow!("Need to return only one value!"))
                                        }
                                    },
                                    "input" => {
                                        let mut userInput = String::new();
                                        let stdin = std::io::stdin();
                                            stdin.read_line(&mut userInput);
                                        Ok(Data::Number(dec!(0)))
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
                                        "Number" => {
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
                                        if let Ok(_returncode) = program.run_loop(writer, shell) {
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
                        parser::Ast::ForLoop {
                            element,
                            elements,
                            block,
                        } => {
                            let collection = elements.evaluate(&self, writer)?;
                            match collection {
                                Data::Array(array) => {
                                    for item in array.iter() {
                                        let mut local_variables = self.variable.clone();
                                        local_variables.insert(element.to_string(), item.clone());
                                        let mut program = Program {
                                            commands: *block.clone(),
                                            current_line: 0,
                                            variable: local_variables,
                                            function: self.function.clone(),
                                            std_commands: self.std_commands.clone(),
                                        };
                                        if program.run_loop(writer, shell).is_err() {
                                            panic!("For loop panicked!");
                                        }
                                        self.variable = program.variable;
                                    }
                                }
                                _ => {
                                    panic!("For loop collection must be an array!");
                                }
                            }
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
            }
            _ => unimplemented!("{:?}", &self.commands),
        }
        Ok(ReturnType::None)
    }
}

trait Evaluate {
    fn evaluate(&self, program: &Program, writer: &mut impl std::io::Write) -> Result<Data>;
}

impl Evaluate for parser::Ast {
    fn evaluate(&self, program: &Program, writer: &mut impl std::io::Write) -> Result<Data> {
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
                    "!" => Ok(Data::Number(
                        Decimal::from_i32(
                            (1..=if f1.is_integer() {
                                f1.to_i32().unwrap()
                            } else {
                                panic!("cannot use decimal value to do factorial")
                            })
                                .product(),
                        )
                        .unwrap(),
                    )),
                    "<" => Ok(Data::Bool(f1 < f2)),
                    ">" => Ok(Data::Bool(f1 > f2)),
                    "<=" => Ok(Data::Bool(f1 <= f2)),
                    ">=" => Ok(Data::Bool(f1 >= f2)),
                    "^" => Ok(Data::Number(f1.powd(f2))),
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
            parser::Ast::ArrayAccess { expr, whereto } => {
                // Evaluate the array expression to get the array
                let array = match expr.evaluate(program, writer)? {
                    Data::Array(a) => a,
                    _ => return Err(anyhow!("Error: expected an array")),
                };

                // Check if the whereto is a single index or a slice
                match &**whereto {
                    // Single index
                    parser::Ast::Int(index) => {
                        // Access the array element at the given index
                        array
                            .get(index.to_usize().unwrap())
                            .cloned()
                            .ok_or_else(|| anyhow!("Error: index out of bounds: {:?}", array))
                    }
                    // Slice
                    parser::Ast::AstSlice { from, to } => {
                        // Evaluate the slice indices expressions to get the start and end indices
                        let start_index =
                            match from.as_ref().map(|expr| expr.evaluate(program, writer)) {
                                Some(Ok(Data::Number(n))) => n.to_usize().unwrap(),
                                Some(Err(e)) => return Err(e),
                                None => 0,
                                _ => unimplemented!(),
                            };
                        let end_index = match to.as_ref().map(|expr| expr.evaluate(program, writer))
                        {
                            Some(Ok(Data::Number(n))) => n.to_usize().unwrap(),
                            Some(Err(e)) => return Err(e),
                            None => array.len() - 1,
                            _ => unimplemented!(),
                        };

                        // Return a slice of the array from start_index to end_index
                        Ok(Data::Array(array[start_index..=end_index].to_vec()))
                    }
                    _ => Err(anyhow!("Error: expected an index or a slice")),
                }
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
                    matchcmd!(id, "Function isn't enabled or it can't be evaluated.", {
                        "return" => {
                            if let Some(arg) = args.first() {
                                let value = arg.evaluate(&program, writer).unwrap();
                                Ok(value)
                            } else {
                                Err(anyhow!("Need to return only one value!"))
                            }
                        },
                        "split" => {
                            if let Some(arg) = args.first() {
                                let value = arg.evaluate(&program, writer).unwrap();
                                let mut x = Vec::new();
                                let mut splitVal = String::from(" ");
                                if let Some(arg) = args.get(1) {
                                    splitVal = arg.evaluate(&program, writer).unwrap().as_string();
                                }
                                for i in value.as_string().trim().split(splitVal.as_str()) {
                                    if let Ok(n) = Decimal::from_str(i) {
                                        x.push(Data::Number(n));
                                    }
                                    else {
                                        match i {
                                            "true" => {x.push(Data::Bool(true))},
                                            "false" => {x.push(Data::Bool(false))},
                                            _ => x.push(Data::String(String::from(i)))
                                        }
                                    }
                                }
                                return Ok(Data::Array(x))
                            }
                            Ok(Data::Number(dec!(1)))
                        },
                        "join" => {
                            if let (Some(left), Some(right)) = (args.get(0), args.get(1)) {
                                if let (Ok(l), Ok(r)) = (left.evaluate(&program, writer), right.evaluate(&program, writer)) {
                                    match (l, r) {
                                        (Data::Array(l), Data::Array(r)) => {return Ok(Data::Array([l, r].concat()))}
                                        _ => panic!("Both argument needs to be an array!")
                                    }
                                }
                                else {
                                    panic!("Unable to parse!")
                                }
                            } else {
                                panic!("Not enough argument!")
                            }
                        },
                        "input" => {
                            let mut userInput = String::new();
                            let stdin = std::io::stdin();
                            stdin.read_line(&mut userInput);
                            Ok(Data::String(userInput))
                        },
                        "inputf" => {
                            if let Some(format_arg) = args.first() {
                                let format_string = format_arg.evaluate(&program, writer).unwrap().as_string();
                                let mut user_input = String::new();

                                // Read user input
                                let stdin = std::io::stdin();
                                stdin.read_line(&mut user_input).expect("Failed to read user input");

                                // Split the format string into individual format specifiers
                                let format_specifiers: Vec<&str> = format_string.trim().split(' ').collect();

                                // Split the user input based on spaces and trim any leading/trailing whitespaces
                                let user_values: Vec<&str> = user_input.trim().split(' ').collect();

                                // Check if the number of format specifiers matches the number of user input values
                                if format_specifiers.len() != user_values.len() {
                                    return Err(anyhow!("Input does not match the specified format"));
                                }

                                // Convert user input values to the corresponding Data types based on format specifiers
                                let mut result = Vec::new();
                                for (i, &format_specifier) in format_specifiers.iter().enumerate() {
                                    match format_specifier {
                                        "%Number" => {
                                            if let Ok(number) = Decimal::from_str(user_values[i]) {
                                                result.push(Data::Number(number));
                                            } else {
                                                return Err(anyhow!("Invalid number format"));
                                            }
                                        },
                                        "%String" => {
                                            result.push(Data::String(String::from(user_values[i])));
                                        },
                                        "%Bool" => {
                                            if let Ok(boolean) = bool::from_str(user_values[i]) {
                                                result.push(Data::Bool(boolean));
                                            } else {
                                                return Err(anyhow!("Invalid boolean format"));
                                            }
                                        },
                                        _ => {
                                            return Err(anyhow!("Invalid format specifier: {}", format_specifier));
                                        }
                                    }
                                }

                                return Ok(Data::Array(result));
                            }
                            Ok(Data::Number(dec!(1)))
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
                                        "Number" => {
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
                            let returncode = program.run_loop(writer, &mut Shell::new());
                            if let Ok(ret) = returncode {
                                match ret {
                                    ReturnType::None => Ok(Data::Number(dec!(0))),
                                    ReturnType::Ok(red) => Ok(red),
                                }
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
