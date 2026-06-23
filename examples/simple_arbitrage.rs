use speculam_solver::{Constraint, ConstraintSystem, SpeculamEngine, SpeculamSolution};

fn main() {
    println!("=== S.P.E.C.U.L.A.M. Example: Simple Arbitrage ===");

    let mut system = ConstraintSystem::new();

    // Pool A (Buy SOL cheap)
    // Initial reserves: X (SOL) = 1,000.0, Y (USDC) = 150,000.0
    let x_a = system.add_variable("X_A", 1000.0, 0.0);
    let y_a = system.add_variable("Y_A", 150000.0, 0.0);
    let k_a = system.add_variable("K_A", 150000000.0, 0.0);

    // Pool B (Sell SOL expensive)
    // Initial reserves: X (SOL) = 1,000.0, Y (USDC) = 160,000.0
    let x_b = system.add_variable("X_B", 1000.0, 0.0);
    let y_b = system.add_variable("Y_B", 160000.0, 0.0);
    let k_b = system.add_variable("K_B", 160000000.0, 0.0);

    // Swap variables (token flows)
    let delta_y = system.add_variable("DeltaY", 1000.0, 1.0);
    let delta_x = system.add_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = system.add_variable("DeltaY_Out", 1000.0, 1.0);

    // Intermediate post-trade states
    let x_a_post = system.add_variable("X_A_Post", 994.0, 1.0);
    let y_a_post = system.add_variable("Y_A_Post", 151000.0, 1.0);

    let x_b_post = system.add_variable("X_B_Post", 1006.0, 1.0);
    let y_b_post = system.add_variable("Y_B_Post", 159000.0, 1.0);

    // Pool A physical constraints (USDC -> SOL)
    system.add_constraint(Constraint::SumEquality {
        name: "pool_a_y".to_string(),
        sumands: vec![y_a, delta_y],
        result: y_a_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "pool_a_x".to_string(),
        sumands: vec![x_a_post, delta_x],
        result: x_a,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "pool_a_amm".to_string(),
        factors: vec![x_a_post, y_a_post],
        result: k_a,
    });

    // Pool B physical constraints (SOL -> USDC)
    system.add_constraint(Constraint::SumEquality {
        name: "pool_b_x".to_string(),
        sumands: vec![x_b, delta_x],
        result: x_b_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "pool_b_y".to_string(),
        sumands: vec![y_b_post, delta_y_out],
        result: y_b,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "pool_b_amm".to_string(),
        factors: vec![x_b_post, y_b_post],
        result: k_b,
    });

    // Target: maximize elastic profit
    let profit_target = system.add_variable("Profit_Target", 50000.0, 0.0);
    let delta_y_plus_target = system.add_variable("DeltaY_Plus_Target", 51000.0, 1.0);

    system.add_constraint(Constraint::SumEquality {
        name: "sum_target".to_string(),
        sumands: vec![delta_y, profit_target],
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
            let dy = adjusted_values.get("DeltaY").unwrap();
            let dy_out = adjusted_values.get("DeltaY_Out").unwrap();
            println!("Arbitrage resolved successfully!");
            println!("USDC Injected: {:.2} USDC", dy);
            println!("USDC Obtained: {:.2} USDC", dy_out);
            println!("Net Profit: {:.2} USDC", dy_out - dy);
        }
        _ => println!("Error optimizing arbitrage."),
    }
}
