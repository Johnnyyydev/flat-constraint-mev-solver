use speculam_solver::{
    Autogenesis, Constraint, ConstraintSystem, NetworkBridge, QuantumJumper, SpeculamEngine,
    SpeculamSolution, StressField,
};

#[tokio::main]
async fn main() {
    // ANSI colors for styling
    let cyan = "\x1b[36;1m";
    let green = "\x1b[32;1m";
    let yellow = "\x1b[33;1m";
    let red = "\x1b[31;1m";
    let reset = "\x1b[0m";

    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!(
        "{}   S.P.E.C.U.L.A.M. v5 - LIVE ASYNCHRONOUS NETWORK BRIDGE   {}",
        cyan, reset
    );
    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!("  Proactive Mirror System for Universal Collapses");
    println!("  and Advanced Matrix Logic in Rust (Tokio Runtime)\n");

    let engine = SpeculamEngine::new();

    // =========================================================================
    // SCENARIO 1: Pure & Rigid Contradiction (10 + 1 = 12)
    // =========================================================================
    println!(
        "{}--- SCENARIO 1: Pure Contradiction (10 + 1 = 12) ---{}",
        yellow, reset
    );
    println!("Fixed variables: A = 10, B = 1, C = 12");
    println!("Constraint: A + B = C (Contiguous Memory)\n");

    let mut system1 = ConstraintSystem::new();
    let a1 = system1.add_variable("A", 10.0, 0.0);
    let b1 = system1.add_variable("B", 1.0, 0.0);
    let c1 = system1.add_variable("C", 12.0, 0.0);

    system1.add_constraint(Constraint::SumEquality {
        name: "base_sum".to_string(),
        sumands: vec![a1, b1],
        result: c1,
    });

    system1.precompute_adjacencies();
    let analysis1 = engine.evaluate(&system1);
    show_result(&analysis1, cyan, green, red, reset);

    // =========================================================================
    // SCENARIO 2: Elastic Relaxation (10 + 1 = X, with X malleable)
    // =========================================================================
    println!(
        "\n{}--- SCENARIO 2: Elastic Relaxation (10 + 1 = X, initial X = 12) ---{}",
        yellow, reset
    );
    println!("Variables: A = 10 (fixed), B = 1 (fixed), X = 12 (MALLEABLE, elasticity = 1.0)\n");

    let mut system2 = ConstraintSystem::new();
    let a2 = system2.add_variable("A", 10.0, 0.0);
    let b2 = system2.add_variable("B", 1.0, 0.0);
    let x2 = system2.add_variable("X", 12.0, 1.0);

    system2.add_constraint(Constraint::SumEquality {
        name: "malleable_sum".to_string(),
        sumands: vec![a2, b2],
        result: x2,
    });

    system2.precompute_adjacencies();
    let analysis2 = engine.evaluate(&system2);
    show_result(&analysis2, cyan, green, red, reset);

    // =========================================================================
    // SCENARIO 3: Cyclic Conflict Graph
    // =========================================================================
    println!(
        "\n{}--- SCENARIO 3: Cyclic Conflict Graph ---{}",
        yellow, reset
    );
    println!("Variables: A = 3.0 (fixed), B = 4.0 (fixed), Z = 5.0 (malleable, elasticity = 1.0)");
    println!("Constraints: A + B = Z  &  Z + B = 10\n");

    let mut system3 = ConstraintSystem::new();
    let a3 = system3.add_variable("A", 3.0, 0.0);
    let b3 = system3.add_variable("B", 4.0, 0.0);
    let const_10 = system3.add_variable("Const_10", 10.0, 0.0);
    let z3 = system3.add_variable("Z", 5.0, 1.0);

    system3.add_constraint(Constraint::SumEquality {
        name: "relation_A_B_Z".to_string(),
        sumands: vec![a3, b3],
        result: z3,
    });

    system3.add_constraint(Constraint::SumEquality {
        name: "relation_Z_B_10".to_string(),
        sumands: vec![z3, b3],
        result: const_10,
    });

    system3.precompute_adjacencies();
    let analysis3 = engine.evaluate(&system3);
    show_result(&analysis3, cyan, green, red, reset);

    // =========================================================================
    // SCENARIO 4: Autogenesis & DePIN Elastic Balance
    // =========================================================================
    println!(
        "\n{}--- SCENARIO 4: Autonomous Autogenesis & Chaotic DePIN Ingestion ---{}",
        yellow, reset
    );
    let raw_flow = r#"
        // Definition of elastic network topology
        fixed(Source_Node, 5.0)
        elastic(Bridge_Node, 2.0, 2.0)
        var(Dest_Node, 15.0)
        fixed(Network_Leak, 1.0)
        var(Exit_Node, 20.0)

        // Physical flow relations
        Source_Node * Bridge_Node = Dest_Node
        Dest_Node + Network_Leak = Exit_Node
        range(Exit_Node, 0.0, 10.0)
    "#;
    println!(
        "Input Flow compiled autonomously by the Automaton:\n{}",
        raw_flow
    );

    match Autogenesis::compile_raw_flow(raw_flow) {
        Ok(system4) => {
            let analysis4 = engine.evaluate(&system4);
            show_result(&analysis4, cyan, green, red, reset);
        }
        Err(e) => {
            println!("{}Autogenesis Error: {}{}", red, e, reset);
        }
    }

    // =========================================================================
    // SCENARIO 5: Discrete Nonce Search (Quantum Jumper)
    // =========================================================================
    println!(
        "\n{}--- SCENARIO 5: Discrete Nonce Search (Quantum Jumper) ---{}",
        yellow, reset
    );
    println!("Objective: Find an integer nonce 'Nonce' such that:");
    println!("  - Nonce * 7.0 = 581.0  => (Discrete mathematical solution: 83)");
    println!(
        "  - 'Nonce' starts at 10.0 and is elastically forced to crystallize as an integer.\n"
    );

    let mut system5 = ConstraintSystem::new();
    let factor = system5.add_variable("Factor", 7.0, 0.0);
    let target = system5.add_variable("Target", 581.0, 0.0);
    let nonce = system5.add_variable("Nonce", 10.0, 1.0); // Malleable

    system5.add_constraint(Constraint::ProductEquality {
        name: "nonce_equation".to_string(),
        factors: vec![nonce, factor],
        result: target,
    });

    let jumper = QuantumJumper::new();
    println!("Starting periodic elastic annealing of discrete coordinates...");

    system5.precompute_adjacencies();
    match jumper.jump_discrete_space(&system5, &[nonce]) {
        Some(values) => {
            println!("{}>>> SUCCESS: QUANTUM JUMP COMPLETED <<<{}", green, reset);
            let nonce_val = values.get("Nonce").unwrap();
            println!("  - Integer 'Nonce' found: {:.1}", nonce_val);

            // Stress verification
            let mut verifier = system5.clone();
            verifier.values[nonce] = *nonce_val;
            let final_stress = StressField::calculate(&verifier);
            println!(
                "  - Residual system energy with integer nonce: {:.4}",
                final_stress.total_energy
            );
        }
        None => {
            println!(
                "{}>>> ERROR: Could not crystallize discrete nonce <<<{}",
                red, reset
            );
        }
    }

    // =========================================================================
    // SCENARIO 6: Hot Asynchronous Network Bridge (Live Ingestion)
    // =========================================================================
    println!(
        "\n{}--- SCENARIO 6: Asynchronous Network Bridge (Live Ingestion & Integration) ---{}",
        yellow, reset
    );
    println!("Compiling base network topology for DePIN routing...");

    let depin_flow = r#"
        fixed(Source_Node, 5.0)
        elastic(Bridge_Node, 2.0, 2.0)
        var(Dest_Node, 15.0)
        fixed(Network_Leak, 1.0)
        var(Exit_Node, 20.0)

        Source_Node * Bridge_Node = Dest_Node
        Dest_Node + Network_Leak = Exit_Node
        range(Exit_Node, 0.0, 10.0)
    "#;

    if let Ok(base_system) = Autogenesis::compile_raw_flow(depin_flow) {
        let rx = NetworkBridge::start_simulated_stream(600); // Updates every 600ms

        println!("Starting continuous background processing (duration: 4 seconds)...");
        let bridge_handle = tokio::spawn(async move {
            NetworkBridge::process_flow(rx, base_system).await;
        });

        // Let telemetry simulation run for 4 seconds
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        bridge_handle.abort();
        println!("\n[MAIN] Asynchronous telemetry stream finished correctly.");
    }

    println!(
        "\n{}=========================================================={}",
        cyan, reset
    );
    println!(
        "{}            END OF S.P.E.C.U.L.A.M. ENGINE ANALYSIS        {}",
        cyan, reset
    );
    println!(
        "{}=========================================================={}",
        cyan, reset
    );
}

fn show_result(solution: &SpeculamSolution, cyan: &str, green: &str, red: &str, reset: &str) {
    match solution {
        SpeculamSolution::Direct { values } => {
            println!(
                "{}>>> DIRECT SOLUTION FOUND (No Tension) <<<{}",
                green, reset
            );
            let mut sorted: Vec<_> = values.iter().collect();
            sorted.sort_by_key(|a| a.0);
            for (var, val) in sorted {
                println!("  - Variable '{}': {:.4}", var, val);
            }
        }
        SpeculamSolution::Hint {
            explanation,
            adjusted_values,
            ..
        } => {
            println!(
                "{}>>> SPECULAM SOLVER: ALTERNATIVE PATH (PHASE COLLAPSE) <<<{}",
                cyan, reset
            );
            println!("{}", explanation);
            println!("Final values in equilibrium:");
            let mut sorted: Vec<_> = adjusted_values.iter().collect();
            sorted.sort_by_key(|a| a.0);
            for (var, val) in sorted {
                println!("  - '{}' = {:.4}", var, val);
            }
        }
        SpeculamSolution::HighComplexity { stress, message } => {
            println!("{}>>> ERROR: COMPLEXITY BEYOND LIMITS <<<{}", red, reset);
            println!("Message: {}", message);
            println!("Residual system energy: {:.4}", stress.total_energy);
        }
    }
    println!();
}
