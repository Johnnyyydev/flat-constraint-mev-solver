use criterion::{Criterion, criterion_group, criterion_main};
use speculam_solver::{Constraint, ConstraintSystem, SpeculamEngine};

fn create_benchmark_system(num_variables: usize) -> ConstraintSystem {
    let mut system = ConstraintSystem::new();
    let mut vars = Vec::new();

    // Add elastic variables
    for i in 0..num_variables {
        let var = system.add_variable(&format!("x_{}", i), i as f64 * 1.5, 1.0);
        vars.push(var);
    }

    // Add sequential sum constraints to generate graph stress
    for i in 0..(num_variables - 1) {
        system.add_constraint(Constraint::SumEquality {
            name: format!("sum_{}", i),
            sumands: vec![vars[i]],
            result: vars[i + 1],
        });
    }

    system.precompute_adjacencies();
    system
}

fn bench_solver(c: &mut Criterion) {
    let system_10 = create_benchmark_system(10);
    let system_100 = create_benchmark_system(100);
    let system_1000 = create_benchmark_system(1000);
    let engine = SpeculamEngine::new();

    c.bench_function("solver_10_vars", |b| b.iter(|| engine.evaluate(&system_10)));

    c.bench_function("solver_100_vars", |b| {
        b.iter(|| engine.evaluate(&system_100))
    });

    c.bench_function("solver_1000_vars", |b| {
        b.iter(|| engine.evaluate(&system_1000))
    });
}

criterion_group!(benches, bench_solver);
criterion_main!(benches);
