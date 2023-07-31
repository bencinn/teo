use criterion::{black_box, criterion_group, criterion_main, Criterion};
use teolang::program::parser::Ast;

fn benchmark_parser(c: &mut Criterion) {
    // Basic input
    let input = r#"
            n = 3;
            n = 2;
            i(n);
            x = [5, 7, 3];
           print(custom_pow(3, 4));
           return(0);
           print("Unreachable??");
        "#;

    c.bench_function("ast_parser", |b| {
        b.iter(|| Ast::parse_code(black_box(input)))
    });
}

criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
