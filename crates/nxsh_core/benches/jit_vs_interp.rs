use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nxsh_core::context::ShellContext;
use nxsh_core::executor::{ExecutionStrategy, Executor};
use nxsh_parser::Parser;

fn bench_jit_vs_interp(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_vs_interp");

    let script = r#"
        a=1
        b=2
        echo $a
        echo $b
        for i in 1 2 3 4 5; do echo $i; done
    "#;

    let parser = Parser::new();
    let ast = parser.parse(script).expect("parse");

    // Interpreter baseline
    group.bench_function("interp_execute", |b| {
        b.iter(|| {
            let mut exec = Executor::new();
            exec.set_strategy(ExecutionStrategy::DirectInterpreter);
            let mut ctx = ShellContext::new();
            let res = exec.execute(&ast, &mut ctx).expect("exec");
            black_box(res.exit_code)
        })
    });

    // MIR/JIT engine
    group.bench_function("mir_execute", |b| {
        b.iter(|| {
            let mut exec = Executor::new();
            exec.set_strategy(ExecutionStrategy::MirEngine);
            let mut ctx = ShellContext::new();
            let res = exec.execute(&ast, &mut ctx).expect("exec");
            black_box(res.exit_code)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_jit_vs_interp);
criterion_main!(benches);
