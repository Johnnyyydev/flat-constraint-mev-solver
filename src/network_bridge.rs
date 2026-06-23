use crate::espejo::{SpeculamEngine, SpeculamSolution};
use crate::grafo::ConstraintSystem;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Async network bridge connecting the elastic solver with real-time telemetry streams.
pub struct NetworkBridge;

impl NetworkBridge {
    /// Starts an async task simulating a continuous feed of DePIN network telemetry.
    /// Emits network fluctuations at the specified interval.
    pub fn start_simulated_stream(interval_ms: u64) -> mpsc::Receiver<(String, f64)> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let events = [
                ("Nodo_Origen".to_string(), 8.0),
                ("Fuga_Red".to_string(), 2.0),
                ("Nodo_Origen".to_string(), 4.0),
                ("Fuga_Red".to_string(), 0.5),
                ("Nodo_Origen".to_string(), 6.5),
                ("Fuga_Red".to_string(), 1.2),
            ];

            let mut idx = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;

                let (var_name, val) = &events[idx % events.len()];
                if tx.send((var_name.clone(), *val)).await.is_err() {
                    break; // Receiver closed, end simulation
                }

                idx += 1;
            }
        });

        rx
    }

    /// Listens to the async stream, updates values in the flat system layout in-place,
    /// and triggers ultra-fast sub-millisecond elastic stress re-evaluations.
    pub async fn process_flow(mut rx: mpsc::Receiver<(String, f64)>, mut system: ConstraintSystem) {
        let engine = SpeculamEngine::new();
        let cyan = "\x1b[36m";
        let green = "\x1b[32m";
        let yellow = "\x1b[33m";
        let reset = "\x1b[0m";

        println!(
            "  [BRIDGE] Listening to async telemetry... ready to dissipate stress in real-time.\n"
        );

        while let Some((var_name, new_value)) = rx.recv().await {
            println!(
                "{}----------------------------------------------------------{}",
                cyan, reset
            );
            println!(
                "  [LIVE TELEMETRY] Ingested: {}{} = {:.2}{}",
                yellow, var_name, new_value, reset
            );

            // 1. O(1) hot path update directly in flat contiguous memory
            if let Some(&var_idx) = system.variable_indices.get(&var_name) {
                let previous_value = system.values[var_idx];
                system.values[var_idx] = new_value;

                println!(
                    "  [FLAT MEMORY] Hot-updated '{}': {:.2} -> {:.2}",
                    var_name, previous_value, new_value
                );

                // 2. Perform parallel sub-millisecond stress re-evaluation
                let t_solve_start = Instant::now();
                let solution = engine.evaluate(&system);
                let t_solve_elapsed = t_solve_start.elapsed();

                println!(
                    "  [SPECULAM SOLVER] Stress re-equilibration completed in: {}{:.2?}{}",
                    green, t_solve_elapsed, reset
                );

                // 3. Print stress dissipation results
                match solution {
                    SpeculamSolution::Hint {
                        adjusted_values,
                        deviations,
                        ..
                    } => {
                        if !deviations.is_empty() {
                            println!("  [DEFORMATIONS]");
                            for (var, delta) in &deviations {
                                println!(
                                    "    - '{}' adjusted by [{:+.4}] -> final: {:.4}",
                                    var,
                                    delta,
                                    adjusted_values.get(var).unwrap()
                                );
                            }
                        } else {
                            println!(
                                "  [STATE] Change fully absorbed by the system without residual deformation."
                            );
                        }
                    }
                    SpeculamSolution::Direct { values } => {
                        println!("  [STATE] Perfect direct equilibrium. Values:");
                        for (var, val) in &values {
                            println!("    - '{}' = {:.4}", var, val);
                        }
                    }
                    SpeculamSolution::HighComplexity { stress, message } => {
                        println!("  [WARNING] High complexity detected: {}", message);
                        println!("    Final system stress: {:.4}", stress.total_energy);
                    }
                }
            } else {
                println!(
                    "  [WARNING] Telemetry ignored: '{}' does not belong to the active logical graph.",
                    var_name
                );
            }
            println!(
                "{}----------------------------------------------------------{}",
                cyan, reset
            );
        }
    }
}
