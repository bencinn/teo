# Teo
Teo is a programming language that was designed for kids.

## Features
- [x] Abstract Syntax Tree
- [x] Data conversion
- [x] String
- [x] Easy-to-modify structure
- [x] Type error detection on runtime
- [x] Performance/Quality-Of-Life
    - [x] Faster parser
    - [x] Getting faster as a whole
- [x] If statement

## Building
```bash
cargo build --release --all-features # use --all-features when you want to enable all features that are not enabled on default (they still have to be enable with --features <feature name>)
./target/release/teo --help
```

## Modifying
To add more commands to the Teo runtime, you can add it on `impl Evaluate for parser::Ast -> match case -> parser::Ast::FunctionCall`. Then, you can add more entires to the `matchcmd!()` macro. This place will be used for evaluation function. If you want to create a normal function, you should use `impl Program -> fn run_loop -> match case -> parser::Ast::FunctionCall`. Remember, after adding to the match case, you need to add to the Cargo.toml's features list as well.

To add more syntax, you could modify the parser at src/program/parser/mod.rs and add another match arm at `impl Program -> fn run_loop -> match case` and `impl Evaluate for parser::Ast -> match case` as well if you want that syntax to be evaluateable.
