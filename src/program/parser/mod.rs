use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt;
use std::rc::Rc;

extern crate pest;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Abstract syntax tree type
/// Should have tree structure but I'm stupid so not now
pub enum Ast {
    /// Code block
    Block(
        /// Map of the block
        Vec<Ast>,
    ),
    /// String data type
    String(String),
    /// Decimal data type (a bit of misleading name)
    Int(Decimal),
    /// All identifiers (a-Z). Used in function name, variable name, etc.
    Identifier(String),
    /// Binary operation
    /// (Can be nested)
    /// # Example
    /// ```rust
    /// # use teolang::program::parser::Ast;
    /// # use rust_decimal_macros::dec;
    /// let a: Ast = Ast::BinaryOp {
    ///     op: "+".to_string(),
    ///     left: Box::new(Ast::Int(dec!(0.1))),
    ///     right: Box::new(Ast::Int(dec!(0.2))),
    /// };
    /// ```
    BinaryOp {
        /// Operator
        op: String,
        /// Left expression
        left: Box<Ast>,
        /// Right expression
        right: Box<Ast>,
    },
    /// Set expression (id = expr)
    Set {
        /// Identifier to set value to
        id: Box<Ast>,
        /// Expression of value to set
        expr: Box<Ast>,
    },
    /// Define function
    FunctionDefinition {
        /// Function name
        id: String,
        /// Function parameter (Params name, Params type)
        params: Vec<(String, String)>,
        /// Code of the function ([`Ast::Block`])
        body: Box<Ast>,
    },
    /// Function call
    FunctionCall {
        /// Name of function to call
        id: String,
        /// Argument given
        args: Vec<Ast>,
    },
    /// Array (is a Vector in Array clothing)
    Array(
        /// Vector to set
        Vec<Ast>,
    ),
    /// Accessing array field
    ArrayCall {
        /// [`Ast::Identifier`] of the array name
        id: String,
        /// Where to access (wrapped in [`Box`])
        k: Box<Ast>,
    },
    ArrayAccess {
        expr: Rc<Ast>,
        whereto: Box<Ast>,
    },
    AstSlice {
        from: Option<Box<Ast>>,
        to: Option<Box<Ast>>,
    },
    /// Boolean data type
    Bool(
        /// Bool to create
        bool,
    ),
    /// If structure
    If {
        /// Condition for the block to be run
        condition: Box<Ast>,
        /// The code block that will run if condition evaluated to true
        block: Box<Ast>,
    },
    ForLoop {
        element: Box<Ast>,
        elements: Box<Ast>,
        block: Box<Ast>,
    },
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ast::String(s) => write!(f, "\"{}\"", s),
            Ast::Int(n) => write!(f, "{}", n),
            Ast::Bool(k) => write!(f, "{}", if *k { "true" } else { "false" }),
            Ast::Array(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    write!(f, "{}", element)?;
                    if i < elements.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Ast::Identifier(s) => write!(f, "{}", s),
            _ => {
                unimplemented!()
            }
        }
    }
}

fn parse_string(s: pest::iterators::Pair<'_, Rule>) -> String {
    let mut str = "".to_string();
    for i in s.into_inner() {
        match i.as_rule() {
            Rule::raw_string => str = str + i.as_str(),
            Rule::unicode_hex => {
                str = str
                    + std::char::from_u32(u32::from_str_radix(i.as_str(), 16).unwrap())
                        .unwrap()
                        .to_string()
                        .as_str()
            }
            Rule::byte => {
                str = str
                    + (i.as_str().parse::<u8>().expect("To parse") as char)
                        .to_string()
                        .as_str()
            }
            _ => unimplemented!("{:?}", i),
        };
    }
    str
}

fn handle_arr(primary: Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut varindex: Option<Box<Ast>> = None;
    let mut wheretoindex: Option<Box<Ast>> = None;
    for i in primary.into_inner().into_iter() {
        match i.as_rule() {
            Rule::indexable_expr => varindex = Some(Box::new(parse_expr(i.into_inner(), pratt))),
            Rule::from_to_index => {
                let mut x: Vec<Ast> = vec![];
                i.into_inner().into_iter().for_each(|f|
                        // x.push(parse_expr(f.into_inner(), pratt))
                        x.push(parse_expr(Pairs::single(f), pratt)));
                if x.len() != 2 {
                    unreachable!()
                } else {
                    wheretoindex = Some(Box::new(Ast::AstSlice {
                        from: Some(Box::new(x[0].clone())),
                        to: Some(Box::new(x[1].clone())),
                    }));
                }
            }
            Rule::from_index => {
                let mut x: Vec<Ast> = vec![];
                i.into_inner().into_iter().for_each(|f|
                        // x.push(parse_expr(f.into_inner(), pratt))
                        x.push(parse_expr(Pairs::single(f), pratt)));
                if x.len() != 1 {
                    unreachable!()
                } else {
                    wheretoindex = Some(Box::new(Ast::AstSlice {
                        from: Some(Box::new(x[0].clone())),
                        to: None,
                    }));
                }
            }
            Rule::to_index => {
                let mut x: Vec<Ast> = vec![];
                i.into_inner().into_iter().for_each(|f|
                        // x.push(parse_expr(f.into_inner(), pratt))
                        x.push(parse_expr(Pairs::single(f), pratt)));
                if x.len() != 1 {
                    unreachable!()
                } else {
                    wheretoindex = Some(Box::new(Ast::AstSlice {
                        from: None,
                        to: Some(Box::new(x[0].clone())),
                    }));
                }
            }
            Rule::index => wheretoindex = Some(Box::new(parse_expr(i.into_inner(), pratt))),
            _ => unreachable!("{:?}", i.as_rule()),
        }
    }
    if let (Some(v), Some(w)) = (varindex.clone(), wheretoindex.clone()) {
        Ast::ArrayAccess {
            expr: v.into(),
            whereto: w,
        }
    } else {
        unreachable!("{:?} {:?}", varindex, wheretoindex)
    }
}

fn handle_array(primary: Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut x = vec![];
    primary.into_inner().into_iter().for_each(|f| {
        f.into_inner()
            .into_iter()
            .for_each(|j| x.push(parse_expr(j.into_inner(), pratt)))
    });
    Ast::Array(x)
}

fn parse_expr(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) -> Ast {
    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::int => Ast::Int(Decimal::from_str_exact(&primary.as_str()).unwrap()),
            Rule::expr => parse_expr(primary.into_inner(), pratt), // from "(" ~ expr ~ ")"
            Rule::command => handle_command(primary, pratt),
            Rule::ident => Ast::Identifier(primary.as_str().trim().to_string()),
            Rule::string => Ast::String(parse_string(primary)),
            Rule::bool => match primary.as_str() {
                "true" => Ast::Bool(true),
                "false" => Ast::Bool(false),
                _ => unreachable!(),
            },
            Rule::array => handle_array(primary, pratt),
            Rule::arr => handle_arr(primary, pratt),
            _ => unreachable!(),
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg => Ast::BinaryOp {
                op: "-".to_string(),
                left: Box::new(Ast::Int(dec![0])),
                right: Box::new(rhs),
            },
            _ => unreachable!(),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            // Rule::fac => (1..lhs + 1).product(),
            Rule::fac => unimplemented!(),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::comparisonop => Ast::BinaryOp {
                op: op.as_str().to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            Rule::add => Ast::BinaryOp {
                op: "+".to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            Rule::sub => Ast::BinaryOp {
                op: "-".to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            Rule::mul => Ast::BinaryOp {
                op: "*".to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            Rule::div => Ast::BinaryOp {
                op: "/".to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            // Rule::pow => (1..rhs + 1).map(|_| lhs).product(),
            Rule::pow => unimplemented!(),
            _ => unreachable!(),
        })
        .parse(pairs)
}
fn handle_block(j: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Vec<Ast> {
    let mut ast = vec![];
    for p in j.into_inner() {
        match p.as_rule() {
            Rule::expr => {
                ast.push(parse_expr(p.into_inner(), &pratt));
            }
            Rule::command => {
                ast.push(handle_command(p, &pratt));
            }
            Rule::set => {
                ast.push(handle_set(p, &pratt));
            }
            Rule::ifs => {
                ast.push(handle_ifs(p, &pratt));
            }
            Rule::def => ast.push(handle_def(p, pratt)),
            Rule::for_loop => ast.push(handle_for_loop(p, pratt)),
            _ => {
                unimplemented!()
            }
        }
    }
    ast
}
fn parse_code(source: &str) -> Result<Vec<Ast>, pest::error::Error<Rule>> {
    let mut ast = vec![];
    let pratt = PrattParser::new()
        .op(Op::infix(Rule::comparisonop, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Right))
        .op(Op::prefix(Rule::neg))
        .op(Op::postfix(Rule::fac));
    let pairs = MyParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for j in pair.into_inner() {
                    match j.as_rule() {
                        Rule::block => ast.append(&mut handle_block(j, &pratt)),
                        Rule::EOI => {}
                        _ => unreachable!(),
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
    Ok(ast)
}
fn handle_for_loop(p: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut element: Option<Ast> = None;
    let mut elements: Option<Ast> = None;
    let mut codeblock: Option<Ast> = None;
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::ident => element = Some(parse_expr(Pairs::single(i), pratt)),
            Rule::expr => elements = Some(parse_expr(Pairs::single(i), pratt)),
            Rule::block => codeblock = Some(Ast::Block(handle_block(i, pratt))),
            Rule::command => codeblock = Some(Ast::Block(vec![handle_command(i, pratt)])),
            _ => unreachable!("{:?}", i.as_rule()),
        }
    }
    // Ast::Bool(true)
    let ast = Ast::ForLoop {
        element: Box::new(element.unwrap()),
        elements: Box::new(elements.unwrap()),
        block: Box::new(codeblock.unwrap()),
    };
    ast
}
fn handle_ifs(p: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut condition = Ast::Bool(true);
    let mut block = vec![];
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::expr => {
                condition = parse_expr(i.into_inner(), pratt);
            }
            Rule::block => {
                block.append(&mut handle_block(i, pratt));
            }
            Rule::command => block.push(handle_command(i, pratt)),
            _ => unreachable!(),
        }
    }
    Ast::If {
        condition: Box::new(condition),
        block: Box::new(Ast::Block(block)),
    }
}
fn handle_def(p: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut ident = "";
    let mut params = vec![];
    let mut body = vec![];
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::ident => ident = i.as_str(),
            Rule::defargs => {
                for j in i.into_inner() {
                    let mut p_name = String::new();
                    let mut p_type = String::new();
                    j.into_inner().into_iter().for_each(|f| match f.as_rule() {
                        Rule::ident => p_name = f.as_str().trim().to_string(),
                        Rule::p_type => p_type = f.as_str().to_string(),
                        _ => unreachable!(),
                    });
                    params.push((p_name, p_type));
                }
            }
            Rule::command => body.push(handle_command(i, pratt)),
            Rule::block => body.append(&mut handle_block(i, pratt)),
            _ => unreachable!(),
        }
    }
    let returnast = Ast::FunctionDefinition {
        id: ident.to_string(),
        params,
        body: Box::new(Ast::Block(body)),
    };
    returnast
}
fn handle_command(p: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut fn_identifier = None;
    let mut args = vec![];
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::ident => fn_identifier = Some(i.as_str()),
            Rule::args => i
                .into_inner()
                .for_each(|f| args.push(parse_expr(f.into_inner(), pratt))),
            _ => unreachable!(),
        }
    }
    if let Some(i) = fn_identifier {
        Ast::FunctionCall {
            id: i.to_string(),
            args,
        }
    } else {
        unreachable!()
    }
}

fn handle_set(p: pest::iterators::Pair<'_, Rule>, pratt: &PrattParser<Rule>) -> Ast {
    let mut x = None;
    let mut y = None;
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::ident => x = Some(Ast::Identifier(i.as_str().trim().to_string())),
            Rule::expr => y = Some(parse_expr(i.into_inner(), pratt)),
            Rule::arr => x = Some(handle_arr(i, pratt)),
            _ => unimplemented!("{:?}", i),
        }
    }
    if let (Some(x), Some(y)) = (x, y) {
        Ast::Set {
            id: Box::new(x),
            expr: Box::new(y),
        }
    } else {
        unreachable!()
    }
}

impl Ast {
    /// Parse code from string
    /// Return [`Err`] if can't parse, else Return [`Result`]<[`Ast`]>
    /// Example
    /// if let Ok(ast) = parse_code(r#"
    ///     let input = r#"
    ///    n = 3;
    ///    n = 2;
    ///    i(n);
    ///    x = [5, 7, 3];
    ///   print(custom_pow(3, 4));
    ///   return(0);
    ///   print("Unreachable??");
    ///  ");
    pub fn parse_code(block: &str) -> Result<Ast> {
        match parse_code(block) {
            Ok(ast) => Ok(Ast::Block(ast)),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
