use speculam_solver::{Constraint, ConstraintSystem, SpeculamEngine, SpeculamSolution};
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
        "{}    S.P.E.C.U.L.A.M. v5 - MEV ARBITRAGE OPTIMIZER          {}",
        cyan, reset
    );
    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!("  Calculating optimal swap size in microseconds...\n");

    // =========================================================================
    // USE CASE: SOL/USDC Liquidity Arbitrage between ORCA and RAYDIUM
    // =========================================================================
    // Pool Initial Reserves:
    // Pool A (Orca) - SOL price is lower (SOL undervalued):
    //   - Reserve X (SOL) = 1,000.0
    //   - Reserve Y (USDC) = 150,000.0 (Implicit price: 150.0 USDC per SOL)
    //   - K_Orca = 150,000,000.0
    //
    // Pool B (Raydium) - SOL price is higher (SOL overvalued):
    //   - Reserve Y (USDC) = 160,000.0
    //   - Reserve X (SOL) = 1,000.0 (Implicit price: 160.0 USDC per SOL)
    //   - K_Raydium = 160,000,000.0

    let mut system = ConstraintSystem::new();

    // 1. Define rigid variables (immutable initial reserves of pools)
    let x_orca = system.add_variable("X_Orca", 1000.0, 0.0);
    let y_orca = system.add_variable("Y_Orca", 150000.0, 0.0);
    let k_orca = system.add_variable("K_Orca", 150000000.0, 0.0);

    let x_raydium = system.add_variable("X_Raydium", 1000.0, 0.0);
    let y_raydium = system.add_variable("Y_Raydium", 160000.0, 0.0);
    let k_raydium = system.add_variable("K_Raydium", 160000000.0, 0.0);

    // 2. Define elastic variables (representing trade flows)
    //   - DeltaY: USDC input to Orca. Initialized at 1000.0 USDC.
    //   - DeltaX: SOL output from Orca & input to Raydium.
    //   - DeltaY_Out: Final USDC output from Raydium.
    let delta_y = system.add_variable("DeltaY", 1000.0, 1.0);
    let delta_x = system.add_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = system.add_variable("DeltaY_Out", 1000.0, 1.0);

    // Intermediate elastic states post-trade
    let x_orca_post = system.add_variable("X_Orca_Post", 994.0, 1.0);
    let y_orca_post = system.add_variable("Y_Orca_Post", 151000.0, 1.0);

    let x_raydium_post = system.add_variable("X_Raydium_Post", 1006.0, 1.0);
    let y_raydium_post = system.add_variable("Y_Raydium_Post", 159000.0, 1.0);

    // 3. Write physical constraints for constant product AMM curves x * y = k
    //
    // Orca: (Y_Orca + DeltaY) = Y_Orca_Post
    system.add_constraint(Constraint::SumEquality {
        name: "orca_sum_y".to_string(),
        sumands: vec![y_orca, delta_y],
        result: y_orca_post,
    });
    // Orca: (X_Orca - DeltaX) = X_Orca_Post  =>  X_Orca_Post + DeltaX = X_Orca
    system.add_constraint(Constraint::SumEquality {
        name: "orca_sub_x".to_string(),
        sumands: vec![x_orca_post, delta_x],
        result: x_orca,
    });
    // Orca: X_Orca_Post * Y_Orca_Post = K_Orca
    system.add_constraint(Constraint::ProductEquality {
        name: "orca_amm".to_string(),
        factors: vec![x_orca_post, y_orca_post],
        result: k_orca,
    });

    // Raydium: (X_Raydium + DeltaX) = X_Raydium_Post
    system.add_constraint(Constraint::SumEquality {
        name: "ray_sum_x".to_string(),
        sumands: vec![x_raydium, delta_x],
        result: x_raydium_post,
    });
    // Raydium: (Y_Raydium - DeltaY_Out) = Y_Raydium_Post  =>  Y_Raydium_Post + DeltaY_Out = Y_Raydium
    system.add_constraint(Constraint::SumEquality {
        name: "ray_sub_y".to_string(),
        sumands: vec![y_raydium_post, delta_y_out],
        result: y_raydium,
    });
    // Raydium: X_Raydium_Post * Y_Raydium_Post = K_Raydium
    system.add_constraint(Constraint::ProductEquality {
        name: "ray_amm".to_string(),
        factors: vec![x_raydium_post, y_raydium_post],
        result: k_raydium,
    });

    // Safety boundary range: no negative swap inputs
    system.add_constraint(Constraint::Range {
        name: "limit_delta_y".to_string(),
        variable: delta_y,
        min: 0.0,
        max: 50000.0,
    });

    // 4. Profit Attractor Constraint (Maximize DeltaY_Out - DeltaY)
    //   Demanding an ambitious target: DeltaY_Out = DeltaY + 50000.0 (earn 50k USDC).
    //   This force guides DeltaY towards the exact optimal trade size
    //   where AMM slippage cancels out the marginal benefit.
    let profit_target = system.add_variable("Profit_Target", 50000.0, 0.0); // Fixed target
    let delta_y_plus_target = system.add_variable("DeltaY_Plus_Target", 51000.0, 1.0);

    system.add_constraint(Constraint::SumEquality {
        name: "sum_target".to_string(),
        sumands: vec![delta_y, profit_target],
        result: delta_y_plus_target,
    });

    // Profit attractor force
    system.add_constraint(Constraint::DirectEquality {
        name: "profit_attractor".to_string(),
        var_a: delta_y_out,
        var_b: delta_y_plus_target,
    });

    // Precompute layout adjacencies
    system.precompute_adjacencies();

    // 5. Execute elastic optimization
    println!(
        "{}---> Searching for optimal swap size with S.P.E.C.U.L.A.M. v5...{}",
        yellow, reset
    );
    let t_solve_start = Instant::now();
    let engine = SpeculamEngine::new();
    let solution = engine.evaluate(&system);
    let t_solve_elapsed = t_solve_start.elapsed();

    println!("{}>>> ARBITRAGE CALCULATION COMPLETED <<<{}", green, reset);
    println!("  Execution time: {:.2?}\n", t_solve_elapsed);

    // 6. Analyze optimal mathematical solution found
    match solution {
        SpeculamSolution::Hint {
            adjusted_values,
            explanation,
            ..
        } => {
            let dx = adjusted_values.get("DeltaX").unwrap();
            let dy = adjusted_values.get("DeltaY").unwrap();
            let dy_out = adjusted_values.get("DeltaY_Out").unwrap();
            let profit = dy_out - dy;

            println!("{}", explanation);

            println!("  [OPTIMAL TRADE FOUND]");
            println!("  - Injected into Orca (USDC): {:.2} USDC", dy);
            println!(
                "  - Withdrawn from Orca (SOL): {:.4} SOL (Effective price: {:.2} USDC/SOL)",
                dx,
                dy / dx
            );
            println!("  - Withdrawn from Raydium (USDC): {:.2} USDC", dy_out);
            println!(
                "  - Estimated net profit: {}${:.2} USDC{}",
                green, profit, reset
            );
            println!("  - Intermediate SOL moved: {:.4} SOL", dx);
        }
        SpeculamSolution::Direct { values } => {
            println!("Direct solution found without tension.");
            println!("Values: {:?}", values);
        }
        SpeculamSolution::HighComplexity { stress, message } => {
            println!("Error during optimization: {}", message);
            println!("Global stress energy: {}", stress.total_energy);
        }
    }

    println!(
        "\n{}=========================================================={}",
        cyan, reset
    );
}
