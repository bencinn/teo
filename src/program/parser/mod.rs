use peg;
use rust_decimal::Decimal;
use std::{collections::BTreeMap, fmt};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Abstract syntax tree type
/// Should have tree structure but I'm stupid so not now
pub enum Ast {
    /// Code block
    /// (This is also used on the top domain / parent tree)
    Block(
        /// Map of the block
        BTreeMap<i64, Ast>,
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
    /// # use teo::program::parser::Ast;
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
peg::parser! {
        grammar ast_parser() for str {
        rule _() =[' ' | '\t' | '\n'] *

        rule integer() -> Ast = n: $(['0' ..= '9'] +("."['0' ..= '9'] +) ?) {
            Ast::Int(Decimal::from_str_exact(n).unwrap())
        }
        rule string() -> Ast = "\"" s: $([^ '"'] *) "\"" {
            Ast::String(s.to_string())
        }
        rule identifier(
        ) -> Ast = s: $(['a' ..= 'z' | 'A' ..= 'Z' | '_']['a' ..= 'z' | 'A' ..= 'Z' | '_' | '0' ..= '9'] *) {
            Ast::Identifier(s.to_string())
        }
        rule array() -> Ast = "[" _ values:(expression() **("," _)) _ "]" {
            Ast::Array(values)
        }
        rule array_call() -> Ast = id: identifier() _ "[" _ k: expression() _ "]" {
            Ast::ArrayCall {
                id: id.to_string(),
                k: Box::new(k),
            }
        }
        rule atom() -> Ast = "true" {
            Ast::Bool(true)
        }
        / "false" {
            Ast::Bool(false)
        }
        / integer(
        ) / string() / identifier() rule assignment_to_elem() -> Ast = id: array_call() _ "=" _ expr: expression() {
            Ast::Set {
                id: Box::new(id),
                expr: Box::new(expr),
            }
        }
        rule assignment() -> Ast = id: identifier() _ "=" _ expr: expression() {
            Ast::Set {
                id: Box::new(Ast::Identifier(id.to_string())),
                expr: Box::new(expr),
            }
        }
        rule function_param() ->(String, String) = id: identifier() _ ":" _ idtype: identifier() {
            (id.to_string(), idtype.to_string())
        }
        rule function(
        ) -> Ast = "def" _ id: identifier(
        ) _ "(" _ params:(function_param() **("," _)) _ ")" _ "{" _ body:(expression() **(_ ";" _)) _ ";" _ "}" {
            let mut block = BTreeMap::new();
            let mut k = 0;
            for i in body {
                block.insert(k, i);
                k+=1;
            }
            Ast::FunctionDefinition {
                id: id.to_string(),
                params,
                body: Box::new(Ast::Block(block)),
            }
        }
        rule function_call() -> Ast = id: identifier() _ "(" _ args:(expression() **("," _)) _ ")" {
            Ast::FunctionCall {
                id: id.to_string(),
                args,
            }
        }
        rule comparison_op() -> String = "<=" {
            "<=".to_string()
        }
        / ">=" {
            ">=".to_string()
        }
        / "==" {
            "==".to_string()
        }
        / "!=" {
            "!=".to_string()
        }
        / "<" {
            "<".to_string()
        }
        / ">" {
            ">".to_string()
        }
        rule comparison() -> Ast = left: term() _ op: comparison_op() _ right: term() {
            Ast::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        / term(
        ) rule ifs(
        ) -> Ast = "if" _ "(" _ comparison: comparison() _ ")" _ "{" _ body:(expression() **(_ ";" _)) _ ";" _ "}" {
            let mut block = BTreeMap::new();
            let mut k = 0;
            for i in body {
                block.insert(k, i);
                k+=1;
            }
            Ast::If {
                condition: Box::new(comparison),
                block: Box::new(Ast::Block(block)),
            }
        }
        rule factor(
        ) -> Ast = ifs(
        ) / assignment_to_elem(
        ) / assignment(
        ) / function() / function_call() / array() / array_call() / atom() / "(" _ expr: expression() _ ")" {
            expr
        }
        rule term() -> Ast = left: factor() _ op: $(['*' | '/']) _ right: term() {
            Ast::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        / left: factor() _ op: $(['+' | '-']) _ right: term() {
            Ast::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        / factor() rule expression() -> Ast = left: comparison() _ op: $(['*' | '/']) _ right: expression() {
            Ast::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        / left: comparison() _ op: $(['+' | '-']) _ right: expression() {
            Ast::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        / comparison() pub rule program() -> Ast = _ exprs:(expression() **(";" _)) _ ";" ? _ {
            let mut tree = BTreeMap::new();
            let mut i = 0;
            for expr in exprs {
                i+=1;
                tree.insert(i, expr);
            }
            Ast::Block(tree)
        }
    }
}

impl Ast {
    /// Parse code from string
    /// Return [`Err`] if can't parse, else Return [`Result`]<[`Ast`]>
    /// # Example
    /// ### Parsing identifier
    /// ```rust
    /// # use teo::program::parser::Ast;
    /// # use rust_decimal_macros::dec;
    /// # use std::collections::BTreeMap;
    /// assert_eq!(Ast::parse_code("x").unwrap(), Ast::Block(BTreeMap::from([(1, Ast::Identifier("x".to_string()))])));
    /// ```
    ///
    /// ### Parsing set
    /// ```rust
    /// # use teo::program::parser::Ast;
    /// # use rust_decimal_macros::dec;
    /// # use std::collections::BTreeMap;
    /// assert_eq!(Ast::parse_code("x = 0;").unwrap(), Ast::Block(BTreeMap::from([(1, Ast::Set {id: Box::new(Ast::Identifier("x".to_string())), expr: Box::new(Ast::Int(dec!(0)))})])));
    /// assert_eq!(Ast::parse_code("x").unwrap(), Ast::Block(BTreeMap::from([(1, Ast::Identifier("x".to_string()))])));
    /// ```

    pub fn parse_code(block: &str) -> Result<Ast> {
        match ast_parser::program(block) {
            Ok(ast) => Ok(ast),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
