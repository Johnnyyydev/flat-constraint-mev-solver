use speculam_solver::{
    Autogenesis, Constraint, ConstraintSystem, QuantumJumper, SpeculamEngine, SpeculamSolution,
};

#[test]
fn test_coherent_system_direct() {
    let mut system = ConstraintSystem::new();
    let x = system.add_variable("x", 5.0, 0.0);
    let y = system.add_variable("y", 10.0, 0.0);
    let z = system.add_variable("z", 15.0, 0.0);

    system.add_constraint(Constraint::SumEquality {
        name: "x_plus_y_eq_z".to_string(),
        sumands: vec![x, y],
        result: z,
    });

    let engine = SpeculamEngine::new();
    system.precompute_adjacencies();
    let solution = engine.evaluate(&system);

    assert!(matches!(solution, SpeculamSolution::Direct { .. }));
}

#[test]
fn test_rigid_contradiction_generates_hint() {
    let mut system = ConstraintSystem::new();
    let a = system.add_variable("A", 10.0, 0.0);
    let b = system.add_variable("B", 1.0, 0.0);
    let c = system.add_variable("C", 12.0, 0.0);

    system.add_constraint(Constraint::SumEquality {
        name: "invalid_sum".to_string(),
        sumands: vec![a, b],
        result: c,
    });

    let engine = SpeculamEngine::new();
    system.precompute_adjacencies();
    let solution = engine.evaluate(&system);

    if let SpeculamSolution::Hint {
        residual_tensions, ..
    } = solution
    {
        let tension = residual_tensions.get("invalid_sum").unwrap();
        assert!((tension - (-1.0)).abs() < 1e-5);
    } else {
        panic!("Expected a phase collapse Hint");
    }
}

#[test]
fn test_elastic_relaxation() {
    let mut system = ConstraintSystem::new();
    let a = system.add_variable("A", 10.0, 0.0);
    let b = system.add_variable("B", 1.0, 0.0);
    let x = system.add_variable("X", 12.0, 1.0);

    system.add_constraint(Constraint::SumEquality {
        name: "relaxable_sum".to_string(),
        sumands: vec![a, b],
        result: x,
    });

    let engine = SpeculamEngine::new();
    system.precompute_adjacencies();
    let solution = engine.evaluate(&system);

    if let SpeculamSolution::Hint {
        adjusted_values,
        deviations,
        ..
    } = solution
    {
        let x_final = adjusted_values.get("X").unwrap();
        assert!((x_final - 11.0).abs() < 1e-4);

        let x_deviation = deviations.get("X").unwrap();
        assert!((x_deviation - (-1.0)).abs() < 1e-4);
    } else {
        panic!("Expected an elastic relaxation");
    }
}

#[test]
fn test_autogenesis_and_resolution() {
    let raw_flow = r#"
        fixed(A, 5.0)
        fixed(B, 3.0)
        var(X, 10.0)
        A * B = X
    "#;

    let system = Autogenesis::compile_raw_flow(raw_flow).unwrap();
    let engine = SpeculamEngine::new();
    let solution = engine.evaluate(&system);

    if let SpeculamSolution::Hint {
        adjusted_values, ..
    } = solution
    {
        let x_final = adjusted_values.get("X").unwrap();
        // X must converge near 5 * 3 = 15
        assert!((x_final - 15.0).abs() < 1e-4);
    } else {
        panic!("Expected X to relax near 15");
    }
}

#[test]
fn test_quantum_jump_resolution() {
    let mut system = ConstraintSystem::new();
    let factor = system.add_variable("Factor", 7.0, 0.0);
    let target = system.add_variable("Target", 581.0, 0.0);
    let nonce = system.add_variable("Nonce", 10.0, 1.0);

    system.add_constraint(Constraint::ProductEquality {
        name: "nonce_equation".to_string(),
        factors: vec![nonce, factor],
        result: target,
    });

    let jumper = QuantumJumper::new();
    system.precompute_adjacencies();
    let solution = jumper.jump_discrete_space(&system, &[nonce]).unwrap();

    let nonce_final = solution.get("Nonce").unwrap();
    assert!((nonce_final - 83.0).abs() < 1e-5);
}

#[test]
fn test_mev_arbitrage_orca_raydium() {
    let mut system = ConstraintSystem::new();

    // Pool A (Orca) - SOL undervalued (150 USDC/SOL)
    let x_orca = system.add_variable("X_Orca", 1000.0, 0.0);
    let y_orca = system.add_variable("Y_Orca", 150000.0, 0.0);
    let k_orca = system.add_variable("K_Orca", 150000000.0, 0.0);

    // Pool B (Raydium) - SOL overvalued (160 USDC/SOL)
    let x_raydium = system.add_variable("X_Raydium", 1000.0, 0.0);
    let y_raydium = system.add_variable("Y_Raydium", 160000.0, 0.0);
    let k_raydium = system.add_variable("K_Raydium", 160000000.0, 0.0);

    // Swap variables
    let delta_y = system.add_variable("DeltaY", 1000.0, 1.0);
    let delta_x = system.add_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = system.add_variable("DeltaY_Out", 1000.0, 1.0);

    let x_orca_post = system.add_variable("X_Orca_Post", 994.0, 1.0);
    let y_orca_post = system.add_variable("Y_Orca_Post", 151000.0, 1.0);

    let x_raydium_post = system.add_variable("X_Raydium_Post", 1006.0, 1.0);
    let y_raydium_post = system.add_variable("Y_Raydium_Post", 159000.0, 1.0);

    // Constraints
    system.add_constraint(Constraint::SumEquality {
        name: "orca_sum_y".to_string(),
        sumands: vec![y_orca, delta_y],
        result: y_orca_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "orca_sub_x".to_string(),
        sumands: vec![x_orca_post, delta_x],
        result: x_orca,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "orca_amm".to_string(),
        factors: vec![x_orca_post, y_orca_post],
        result: k_orca,
    });

    system.add_constraint(Constraint::SumEquality {
        name: "ray_sum_x".to_string(),
        sumands: vec![x_raydium, delta_x],
        result: x_raydium_post,
    });
    system.add_constraint(Constraint::SumEquality {
        name: "ray_sub_y".to_string(),
        sumands: vec![y_raydium_post, delta_y_out],
        result: y_raydium,
    });
    system.add_constraint(Constraint::ProductEquality {
        name: "ray_amm".to_string(),
        factors: vec![x_raydium_post, y_raydium_post],
        result: k_raydium,
    });

    // Safety range
    system.add_constraint(Constraint::Range {
        name: "limit_delta_y".to_string(),
        variable: delta_y,
        min: 0.0,
        max: 50000.0,
    });

    // Attractor
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

    if let SpeculamSolution::Hint {
        adjusted_values, ..
    } = solution
    {
        let dy = adjusted_values.get("DeltaY").unwrap();
        let dy_out = adjusted_values.get("DeltaY_Out").unwrap();
        let profit = dy_out - dy;

        assert!(
            profit > 50.0,
            "Optimized profit should be substantial (> 50 USDC)"
        );
        assert!(profit < 5000.0, "Profit should be bounded by pool slippage");
        assert!((dy - 950.92).abs() < 50.0);
    } else {
        panic!("Expected elastic Hint solution");
    }
}

#[test]
fn test_triangular_arbitrage_multi_hop() {
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

    // Swap variables
    let delta_y_in = system.add_variable("DeltaY_In", 1000.0, 1.0);
    let delta_x1 = system.add_variable("DeltaX1", 6.0, 1.0);
    let delta_z2 = system.add_variable("DeltaZ2", 60000.0, 1.0);
    let delta_y_out = system.add_variable("DeltaY_Out", 1000.0, 1.0);

    // Post-reserves
    let x1_post = system.add_variable("X1_Post", 994.0, 1.0);
    let y1_post = system.add_variable("Y1_Post", 151000.0, 1.0);

    let x2_post = system.add_variable("X2_Post", 1006.0, 1.0);
    let z2_post = system.add_variable("Z2_Post", 9940000.0, 1.0);

    let z3_post = system.add_variable("Z3_Post", 10060000.0, 1.0);
    let y3_post = system.add_variable("Y3_Post", 159000.0, 1.0);

    // Constraints Pool 1
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

    // Constraints Pool 2
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

    // Constraints Pool 3
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

    // Attractor
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

    if let SpeculamSolution::Hint {
        adjusted_values, ..
    } = solution
    {
        let dy_in = adjusted_values.get("DeltaY_In").unwrap();
        let dy_out = adjusted_values.get("DeltaY_Out").unwrap();
        let profit = dy_out - dy_in;

        assert!(
            profit > 10.0,
            "Triangular profit should be positive (> 10 USDC)"
        );
        assert!(profit < 5000.0);
    } else {
        panic!("Expected Hint solution for triangular arbitrage");
    }
}
