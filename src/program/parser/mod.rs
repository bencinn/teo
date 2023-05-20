use peg;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Ast {
    String(String),
    Int(i64),
    Identifier(String),
    BinaryOp {
        op: String,
        left: Box<Ast>,
        right: Box<Ast>,
    },
    Set {
        id: Box<Ast>,
        expr: Box<Ast>,
    },
    FunctionDefinition {
        id: String,
        params: Vec<(String, String)>,
        body: Vec<Ast>,
    },
    FunctionCall {
        id: String,
        args: Vec<Ast>,
    },
    Array(Vec<Ast>),
    ArrayCall {
        id: String,
        k: Box<Ast>,
    },
    Bool(bool),
}
impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ast::String(s) => write!(f, "\"{}\"", s),
            Ast::Int(n) => write!(f, "{}", n),
            Ast::Identifier(s) => write!(f, "{}", s),
            Ast::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", op, left, right)
            }
            Ast::Set { id, expr } => {
                write!(f, "{} = {}", id, expr)
            }
            Ast::FunctionDefinition { id, params, body } => {
                write!(f, "def {}(", id)?;
                for (i, (param, ty)) in params.iter().enumerate() {
                    write!(f, "{}: {}", param, ty)?;
                    if i < params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                writeln!(f, ") {{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            }
            Ast::FunctionCall { id, args } => {
                write!(f, "{}(", id)?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
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
            Ast::ArrayCall { id, k } => {
                write!(f, "ArrayCall({}, {})", id, k)
            }
            Ast::Bool(k) => write!(f, "{}", if *k { "true" } else { "false" }),
        }
    }
}

peg::parser! {
    grammar ast_parser() for str {
        rule _() = [' '| '\t'| '\n']*
        rule integer() -> Ast = n:$(['0'..='9']+) { Ast::Int(n.parse().unwrap()) }
        rule string() -> Ast = "\"" s:$([^'"']*) "\"" { Ast::String(s.to_string()) }
        rule identifier() -> Ast = s:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) { Ast::Identifier(s.to_string()) }
        rule array() -> Ast
            = "[" _ values:(expression() ** ("," _)) _ "]"
            { Ast::Array(values) }
        rule array_call() -> Ast
            = id:identifier() _ "[" _ k:expression() _ "]"
            { Ast::ArrayCall { id: id.to_string(), k: Box::new(k) } }

        rule atom() -> Ast =
            "true" { Ast::Bool(true) } /
            "false" { Ast::Bool(false) } /
            integer() /
            string() /
            identifier()

        rule assignment_to_elem() -> Ast = id:array_call() _ "=" _ expr:expression() { Ast::Set{ id: Box::new(id), expr: Box::new(expr) } }
        rule assignment() -> Ast = id:identifier() _ "=" _ expr:expression() { Ast::Set{ id: Box::new(Ast::Identifier(id.to_string())), expr: Box::new(expr) } }
        rule function_param() -> (String, String) = id:identifier() _ ":" _ idtype:identifier() {(id.to_string(), idtype.to_string())}
        rule function() -> Ast
            = "def" _ id:identifier() _ "(" _ params:(function_param() ** ("," _)) _ ")" _ "{" _ body:(expression() ** (_ ";" _)) _ ";" _ "}"
            {Ast::FunctionDefinition { id: id.to_string(), params, body}}
        rule function_call() -> Ast
            = id:identifier() _ "(" _ args:(expression() ** ("," _)) _ ")"
            {Ast::FunctionCall {id: id.to_string(), args,}
        }

        rule factor() -> Ast
            = assignment_to_elem() /
            assignment() /
            function() /
            function_call() /
            array() /
            array_call() /
            atom() /
            "(" _ expr:expression() _ ")" { expr }

        rule term() -> Ast
            = left:factor() _ op:$(['*' | '/']) _ right:term()
                { Ast::BinaryOp{ op: op.to_string(), left: Box::new(left), right: Box::new(right) } } /
                factor()

        rule expression() -> Ast = left:term() _ op:$(['+' | '-']) _ right:expression()
                                        { Ast::BinaryOp{ op: op.to_string(), left: Box::new(left), right: Box::new(right) } } /
                                        term()
        pub rule program() -> Vec<Ast> = _ exprs:(expression() ** (";" _)) _ ";"? _ { exprs }
        }
}
impl Ast {
    pub fn parse_code(block: &str) -> Result<Vec<Ast>, peg::error::ParseError<peg::str::LineCol>> {
        match ast_parser::program(block) {
            Ok(ast) => {
                println!("{:#?}", ast);
                Ok(ast)
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }
}
