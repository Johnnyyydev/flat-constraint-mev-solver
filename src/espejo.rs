use crate::estres::StressField;
use crate::grafo::{Constraint, ConstraintSystem};
use std::collections::HashMap;

/// States of the solution returned by the S.P.E.C.U.L.A.M. elastic solver.
#[derive(Debug, Clone)]
pub enum SpeculamSolution {
    /// The system is completely coherent with no significant initial stress.
    Direct {
        /// Map of variable names to their final values.
        values: HashMap<String, f64>,
    },
    /// The system contained contradictions or stresses. Elastic adjustments
    /// have been made to variables or phase collapses proposed to balance it.
    Hint {
        /// Map of variable names to their original input values.
        original_values: HashMap<String, f64>,
        /// Map of variable names to their new balanced values.
        adjusted_values: HashMap<String, f64>,
        /// Map of deviations applied to elastic variables.
        deviations: HashMap<String, f64>,
        /// Map of rigid constraints and their calculated residual stress/tension.
        residual_tensions: HashMap<String, f64>,
        /// Readable heuristic explanation of the solution or inconsistencies.
        explanation: String,
    },
    /// The complexity or contradictions were too high to find a stable local minimum
    /// within the maximum allowed steps.
    HighComplexity {
        /// Final stress field of the solver.
        stress: StressField,
        /// Descriptive message with the cause of complexity or failure.
        message: String,
    },
}

/// Elastic optimization engine that dissipates local stress using gradient descent.
pub struct SpeculamEngine {
    /// Learning rate step for gradient descent.
    pub learning_rate: f64,
    /// Maximum iterations allowed for the solver loop.
    pub max_steps: usize,
    /// Residual energy tolerance below which the system is considered resolved.
    pub tolerance: f64,
    /// Strict execution time budget in microseconds (optional).
    pub max_duration_microseconds: Option<u64>,
}

impl Default for SpeculamEngine {
    fn default() -> Self {
        SpeculamEngine {
            learning_rate: 0.01,
            max_steps: 1000,
            tolerance: 1e-10, // Fine tolerance to balance residual stress
            max_duration_microseconds: None, // No limit by default to prevent debug/test timeouts
        }
    }
}

impl SpeculamEngine {
    /// Creates a new instance of `SpeculamEngine` with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the flat system and searches for alternative paths using elastic relaxation.
    pub fn evaluate(&self, system: &ConstraintSystem) -> SpeculamSolution {
        let initial_stress = StressField::calculate(system);

        // If initial energy is almost zero, the system is already coherent.
        if initial_stress.total_energy < self.tolerance || initial_stress.total_energy.is_nan() {
            return SpeculamSolution::Direct {
                values: system.map_values(),
            };
        }

        let mut working_system = system.clone();
        let mut best_energy = initial_stress.total_energy;
        let mut best_values = working_system.values.clone();
        let mut steps_without_improvement = 0;
        let t_start = std::time::Instant::now();

        // Fast optimization loop using direct memory indexing
        for _ in 0..self.max_steps {
            // Real-time duration control (microseconds)
            if self
                .max_duration_microseconds
                .is_some_and(|limit| t_start.elapsed().as_micros() as u64 >= limit)
            {
                break;
            }

            let current_stress = StressField::calculate(&working_system);

            // Defense against NaN / Infinity in energy
            if current_stress.total_energy.is_nan() || current_stress.total_energy.is_infinite() {
                break;
            }

            if current_stress.total_energy < self.tolerance {
                best_values = working_system.values.clone();
                break;
            }

            if current_stress.total_energy < best_energy {
                best_energy = current_stress.total_energy;
                best_values = working_system.values.clone();
                steps_without_improvement = 0;
            } else {
                steps_without_improvement += 1;
                if steps_without_improvement > 50 {
                    break;
                }
            }

            // Apply gradient to elastic variables with component-wise gradient damping (clipping)
            // to prevent large-scale variables (e.g., AMM invariants) from freezing the rest of the system.
            let len = working_system.values.len();
            let mut detected_nan = false;

            for idx in 0..len {
                if !working_system.is_fixed(idx) {
                    let grad_val = current_stress.gradient[idx];

                    if grad_val.is_nan() || grad_val.is_infinite() {
                        detected_nan = true;
                        break;
                    }

                    let max_g = 10.0;
                    let clipped_g = if grad_val.abs() > max_g {
                        grad_val.signum() * max_g
                    } else {
                        grad_val
                    };
                    let delta = -self.learning_rate * working_system.elasticities[idx] * clipped_g;
                    working_system.values[idx] += delta;
                }
            }

            if detected_nan {
                break;
            }
        }

        // Restore the best stable state found (with lowest energy)
        working_system.values = best_values;
        let final_stress = StressField::calculate(&working_system);

        let original_values = system.map_values();
        let adjusted_values = working_system.map_values();

        let mut deviations = HashMap::new();
        for (k, v_orig) in &original_values {
            let v_ajust = adjusted_values.get(k).unwrap_or(v_orig);
            let diff = v_ajust - v_orig;
            if diff.abs() > 1e-5 {
                deviations.insert(k.clone(), diff);
            }
        }

        let mut residual_tensions = HashMap::new();
        for (r_idx, &tension) in final_stress.tensions.iter().enumerate() {
            if tension.abs() > 1e-5 {
                let name_r = system.constraints[r_idx].name().to_string();
                residual_tensions.insert(name_r, tension);
            }
        }

        // Build a readable heuristic explanation
        let mut explanation = String::new();
        explanation.push_str("--- ELASTIC SPRING RESOLUTION ANALYSIS (S.P.E.C.U.L.A.M. v5) ---\n");

        if !deviations.is_empty() {
            explanation.push_str("• Elastic deviations applied to variables:\n");
            for (var, delta) in &deviations {
                let orig = original_values.get(var).unwrap();
                let new_val = adjusted_values.get(var).unwrap();
                explanation.push_str(&format!(
                    "  - Variable '{}': {:.4} -> {:.4} (deviation of [{:+.4}])\n",
                    var, orig, new_val, delta
                ));
            }
        }

        if !residual_tensions.is_empty() {
            explanation.push_str("• Structural clues for unresolved rigid constraints:\n");
            for (constraint_name, tension) in &residual_tensions {
                if let Some(rest) = system
                    .constraints
                    .iter()
                    .find(|r| r.name() == constraint_name)
                {
                    match rest {
                        Constraint::SumEquality {
                            sumands, result, ..
                        } => {
                            let names_sumands: Vec<String> = sumands
                                .iter()
                                .map(|&idx| system.names[idx].clone())
                                .collect();
                            let name_res = &system.names[*result];

                            let sum_real: f64 =
                                sumands.iter().map(|&idx| working_system.values[idx]).sum();
                            let expected_res = working_system.values[*result];

                            explanation.push_str(&format!(
                                "  - Constraint '{}': ({} = {}) failed. Actual sum: {}, expected: {}.\n",
                                constraint_name, names_sumands.join(" + "), name_res, sum_real, expected_res
                            ));
                            explanation.push_str(&format!(
                                "    >>> PHASE COLLAPSE: To balance the equation, an adjustment of [{:+.4}] is required.\n",
                                -tension
                            ));
                            explanation.push_str(&format!(
                                "    >>> HIDDEN PATH: ({}) {:+.4} = {}\n",
                                sumands
                                    .iter()
                                    .map(|&idx| format!("{}", working_system.values[idx]))
                                    .collect::<Vec<_>>()
                                    .join(" + "),
                                -tension,
                                expected_res
                            ));
                        }
                        Constraint::ProductEquality {
                            factors, result, ..
                        } => {
                            let names_factors: Vec<String> = factors
                                .iter()
                                .map(|&idx| system.names[idx].clone())
                                .collect();
                            let name_res = &system.names[*result];

                            let prod_real: f64 = factors
                                .iter()
                                .map(|&idx| working_system.values[idx])
                                .product();
                            let expected_res = working_system.values[*result];

                            explanation.push_str(&format!(
                                "  - Constraint '{}': ({} = {}) failed. Actual product: {}, expected: {}.\n",
                                constraint_name, names_factors.join(" * "), name_res, prod_real, expected_res
                            ));
                            explanation.push_str(&format!(
                                "    >>> PHASE COLLAPSE: To balance the equation, an adjustment of [{:+.4}] is required.\n",
                                -tension
                            ));
                        }
                        Constraint::Range {
                            variable, min, max, ..
                        } => {
                            let name_var = &system.names[*variable];
                            let val = working_system.values[*variable];
                            explanation.push_str(&format!(
                                "  - Range Constraint '{}': '{}' with value {} is out of boundary [{}, {}].\n",
                                constraint_name, name_var, val, min, max
                            ));
                            explanation.push_str(&format!(
                                "    >>> PHASE COLLAPSE: Shift '{}' by [{:+.4}] to satisfy the boundary.\n",
                                name_var, -tension
                            ));
                        }
                        Constraint::DirectEquality { var_a, var_b, .. } => {
                            let name_a = &system.names[*var_a];
                            let name_b = &system.names[*var_b];
                            let val_a = working_system.values[*var_a];
                            let val_b = working_system.values[*var_b];
                            explanation.push_str(&format!(
                                "  - Direct Equality '{}': '{}' ({}) != '{}' ({}).\n",
                                constraint_name, name_a, val_a, name_b, val_b
                            ));
                            explanation.push_str(&format!(
                                "    >>> PHASE COLLAPSE: Force symmetry collapse with deviation of [{:+.4}].\n",
                                -tension
                            ));
                        }
                    }
                }
            }
        }

        let diverging = final_stress.total_energy.is_nan()
            || final_stress.total_energy.is_infinite()
            || final_stress.total_energy > initial_stress.total_energy * 2.0;

        if diverging {
            SpeculamSolution::HighComplexity {
                stress: final_stress,
                message: "The stress field diverged. Unstable reduction or numerical overflow."
                    .to_string(),
            }
        } else {
            SpeculamSolution::Hint {
                original_values,
                adjusted_values,
                deviations,
                residual_tensions,
                explanation,
            }
        }
    }
}
