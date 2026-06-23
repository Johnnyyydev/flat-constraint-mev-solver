use criterion::{Criterion, criterion_group, criterion_main};
use speculam_solver::{MotorSpeculam, Restriccion, SistemaRestricciones};

fn crear_sistema_benchmark(num_variables: usize) -> SistemaRestricciones {
    let mut sistema = SistemaRestricciones::new();
    let mut vars = Vec::new();

    // Agregar variables elásticas
    for i in 0..num_variables {
        let var = sistema.agregar_variable(&format!("x_{}", i), i as f64 * 1.5, 1.0);
        vars.push(var);
    }

    // Añadir restricciones de suma secuenciales para generar estrés en el grafo
    for i in 0..(num_variables - 1) {
        sistema.agregar_restriccion(Restriccion::IgualdadSuma {
            nombre: format!("suma_{}", i),
            sumandos: vec![vars[i]],
            resultado: vars[i + 1],
        });
    }

    sistema.precalcular_adyacencias();
    sistema
}

fn bench_solver(c: &mut Criterion) {
    let sistema_10 = crear_sistema_benchmark(10);
    let sistema_100 = crear_sistema_benchmark(100);
    let sistema_1000 = crear_sistema_benchmark(1000);
    let motor = MotorSpeculam::new();

    c.bench_function("solver_10_vars", |b| b.iter(|| motor.evaluar(&sistema_10)));

    c.bench_function("solver_100_vars", |b| {
        b.iter(|| motor.evaluar(&sistema_100))
    });

    c.bench_function("solver_1000_vars", |b| {
        b.iter(|| motor.evaluar(&sistema_1000))
    });
}

criterion_group!(benches, bench_solver);
criterion_main!(benches);
