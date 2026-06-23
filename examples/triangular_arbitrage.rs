use speculam_solver::{Constraint, ConstraintSystem, SpeculamEngine, SpeculamSolution};

fn main() {
    println!("=== S.P.E.C.U.L.A.M. Example: Triangular Arbitrage ===");

    let mut system = ConstraintSystem::new();

    // Pool 1: SOL/USDC (USDC -> SOL) - 1,000 SOL / 150,000 USDC
    let x1 = system.add_variable("X1", 1000.0, 0.0);
    let y1 = system.add_variable("Y1", 150000.0, 0.0);
    let k1 = system.add_variable("K1", 150000000.0, 0.0);

    // Pool 2: SOL/BONK (SOL -> BONK) - 1,000 SOL / 10,000,000 BONK
    let x2 = system.add_variable("X2", 1000.0, 0.0);
    let z2 = system.add_variable("Z2", 10000000.0, 0.0);
    let k2 = system.add_variable("K2", 10000000000.0, 0.0);

    // Pool 3: BONK/USDC (BONK -> USDC) - 10,000,000 BONK / 160,000 USDC
    let z3 = system.add_variable("Z3", 10000000.0, 0.0);
    let y3 = system.add_variable("Y3", 160000.0, 0.0);
    let k3 = system.add_variable("K3", 1600000000000.0, 0.0);

    // Swap variables (flows)
    let delta_y_in = system.add_variable("DeltaY_In", 1000.0, 1.0);
    let delta_x1 = system.add_variable("DeltaX1", 6.0, 1.0);
    let delta_z2 = system.add_variable("DeltaZ2", 60000.0, 1.0);
    let delta_y_out = system.add_variable("DeltaY_Out", 1000.0, 1.0);

    // Post-swap reserves
    let x1_post = system.add_variable("X1_Post", 994.0, 1.0);
    let y1_post = system.add_variable("Y1_Post", 151000.0, 1.0);

    let x2_post = system.add_variable("X2_Post", 1006.0, 1.0);
    let z2_post = system.add_variable("Z2_Post", 9940000.0, 1.0);

    let z3_post = system.add_variable("Z3_Post", 10060000.0, 1.0);
    let y3_post = system.add_variable("Y3_Post", 159000.0, 1.0);

    // Pool 1 (USDC -> SOL)
    system.add_constraint(Constraint::SumEquality {
        name: "p1_sum_y".to_string(),
        sumands: vec![y1, delta_y_in],
        result: y1_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "p1_sub_x".to_string(),
        sumands: vec![x1_post, delta_x1],
        result: x1,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "p1_amm".to_string(),
        factors: vec![x1_post, y1_post],
        result: k1,
    });

    // Pool 2 (SOL -> BONK)
    system.add_constraint(Constraint::SumEquality {
        name: "p2_sum_x".to_string(),
        sumands: vec![x2, delta_x1],
        result: x2_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "p2_sub_z".to_string(),
        sumands: vec![z2_post, delta_z2],
        result: z2,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "p2_amm".to_string(),
        factors: vec![x2_post, z2_post],
        result: k2,
    });

    // Pool 3 (BONK -> USDC)
    system.add_constraint(Constraint::SumEquality {
        name: "p3_sum_z".to_string(),
        sumands: vec![z3, delta_z2],
        result: z3_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "p3_sub_y".to_string(),
        sumands: vec![y3_post, delta_y_out],
        result: y3,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "p3_amm".to_string(),
        factors: vec![z3_post, y3_post],
        result: k3,
    });

    // Target profit attractor (Maximize DeltaY_Out - DeltaY_In)
    let profit_target = system.add_variable("Profit_Target", 50000.0, 0.0);
    let delta_y_plus_target = system.add_variable("DeltaY_Plus_Target", 51000.0, 1.0);

    system.add_constraint(Constraint::SumEquality {
        name: "sum_target".to_string(),
        sumands: vec![delta_y_in, profit_target],
        result: delta_y_plus_target,
    });

    system.add_constraint(Constraint::DirectEquality {
        name: "profit_attractor".to_string(),
        var_a: delta_y_out,
        var_b: delta_y_plus_target,
    });

    system.precompute_adjacencies();

    let engine = SpeculamEngine::new();
    let solution = engine.evaluate(&system);

    match solution {
        SpeculamSolution::Hint {
            adjusted_values, ..
        } => {
            let dy_in = adjusted_values.get("DeltaY_In").unwrap();
            let dy_out = adjusted_values.get("DeltaY_Out").unwrap();
            println!("Triangular arbitrage resolved successfully!");
            println!("USDC Input: {:.2} USDC", dy_in);
            println!("USDC Output: {:.2} USDC", dy_out);
            println!("Net Profit: {:.2} USDC", dy_out - dy_in);
        }
        _ => println!("Error optimizing triangular arbitrage."),
    }
}
