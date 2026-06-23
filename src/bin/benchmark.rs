use speculam_solver::{Autogenesis, SpeculamEngine, SpeculamSolution, StressField};
use std::time::Instant;

fn main() {
    let cyan = "\x1b[36;1m";
    let green = "\x1b[32;1m";
    let yellow = "\x1b[33;1m";
    let reset = "\x1b[0m";

    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!(
        "{}       S.P.E.C.U.L.A.M. INDUSTRIAL BENCHMARK (Phase 1)     {}",
        cyan, reset
    );
    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!("  Generating massive synthetic network of variables and constraints...\n");

    let num_variables = 10000;
    let num_blocks = 3300; // Each block defines 3 variables and 3 crossed constraints

    // 1. Generate raw text flow for Autogenesis in memory
    let t_gen_start = Instant::now();
    let mut raw_data = String::new();

    // Variable definition
    for i in 0..num_variables {
        if i % 10 == 0 {
            // Every 10 variables, fix one to act as a boundary condition
            raw_data.push_str(&format!("fixed(V_{}, {}.0)\n", i, (i % 5) + 1));
        } else if i % 10 == 3 {
            // Some variables with custom elasticity
            raw_data.push_str(&format!("elastic(V_{}, 1.0, 1.5)\n", i));
        } else {
            // Rest are standard elastic variables
            raw_data.push_str(&format!("var(V_{}, 0.0)\n", i));
        }
    }

    // Interconnected constraints in cascade (to form a global stress field)
    for b in 0..num_blocks {
        let v0 = b * 3;
        let v1 = b * 3 + 1;
        let v2 = b * 3 + 2;
        let v_next = (b * 3 + 3) % num_variables;

        if v2 < num_variables {
            // Sum Equation: V0 + V1 = V2
            raw_data.push_str(&format!("V_{} + V_{} = V_{}\n", v0, v1, v2));
            // Product Equation: V1 * V2 = V_next
            raw_data.push_str(&format!("V_{} * V_{} = V_{}\n", v1, v2, v_next));
            // Range Constraint on intermediate variable
            raw_data.push_str(&format!("rango(V_{}, 0.0, 50.0)\n", v1));
        }
    }

    let t_gen_elapsed = t_gen_start.elapsed();
    println!(
        "• Generated synthetic data: {} variables and {} equations in plain text.",
        num_variables,
        num_blocks * 3
    );
    println!("  Time taken: {:.2?}\n", t_gen_elapsed);

    // 2. Compile with Autogenesis
    println!(
        "{}---> Starting Autogenesis Compilation...{}",
        yellow, reset
    );
    let t_comp_start = Instant::now();
    let system = match Autogenesis::compile_raw_flow(&raw_data) {
        Ok(s) => s,
        Err(e) => {
            println!("Error compiling graph: {}", e);
            return;
        }
    };
    let t_comp_elapsed = t_comp_start.elapsed();
    println!("{}>>> COMPILATION COMPLETED <<<{}", green, reset);
    println!(
        "  Variables loaded in flat memory layout: {}",
        system.values.len()
    );
    println!("  Constraints compiled: {}", system.constraints.len());
    println!("  Compilation time: {:.2?}\n", t_comp_elapsed);

    // Calculate initial stress
    let initial_stress = StressField::calculate(&system);
    println!(
        "• Initial system elastic energy: {:.4}",
        initial_stress.total_energy
    );

    // 3. Solve with Speculam flat engine
    println!(
        "\n{}---> Running flat matrix solver (Speculam Engine)...{}",
        yellow, reset
    );
    let engine = SpeculamEngine::new();
    let t_solve_start = Instant::now();
    let solution = engine.evaluate(&system);
    let t_solve_elapsed = t_solve_start.elapsed();

    println!("{}>>> RESOLUTION COMPLETED <<<{}", green, reset);
    println!("  Solver execution time: {:.2?}", t_solve_elapsed);

    // Evaluate residual energy
    match solution {
        SpeculamSolution::Hint {
            residual_tensions,
            adjusted_values,
            ..
        } => {
            let mut verifier = system.clone();
            for (var, val) in adjusted_values {
                if let Some(&idx) = verifier.variable_indices.get(&var) {
                    verifier.values[idx] = val;
                }
            }
            let final_stress = StressField::calculate(&verifier);
            println!("• Final elastic energy: {:.6}", final_stress.total_energy);
            println!(
                "  Active unresolved equations (due to rigidity): {}",
                residual_tensions.len()
            );
            println!(
                "  Stress reduction: {:.2}%",
                (1.0 - final_stress.total_energy / initial_stress.total_energy) * 100.0
            );
        }
        SpeculamSolution::Direct { .. } => {
            println!("• Final elastic energy: 0.000000 (Perfect direct solution).");
        }
        SpeculamSolution::HighComplexity { stress, message } => {
            println!("• Exit state: High Complexity");
            println!("  Message: {}", message);
            println!("  Final elastic energy: {:.6}", stress.total_energy);
        }
    }

    println!(
        "\n{}=========================================================={}",
        cyan, reset
    );
}
