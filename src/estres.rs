use crate::grafo::{Constraint, ConstraintSystem};
use rayon::prelude::*;

/// Represents the global stress field state at a given instant using zero-allocation flat arrays.
#[derive(Debug, Clone)]
pub struct StressField {
    /// Total elastic potential energy stored in the system (sum of squared tensions).
    pub total_energy: f64,
    /// Flat vector of tensions aligned with the indices of `system.constraints`.
    /// Eliminates heap allocations in the hot path, enabling SIMD and Rayon parallelization.
    pub tensions: Vec<f64>,
    /// Energy gradient with respect to each variable index.
    pub gradient: Vec<f64>,
}

impl StressField {
    /// Calculates the current stress field of the constraint system.
    ///
    /// This method evaluates the total energy, local tensions of each constraint,
    /// and the local gradient vector in parallel using Rayon.
    pub fn calculate(system: &ConstraintSystem) -> Self {
        let use_parallel = system.constraints.len() > 250 || system.values.len() > 250;

        // Pass 1: Calculate local tensions for each constraint
        let tensions: Vec<f64> = if use_parallel {
            system
                .constraints
                .par_iter()
                .map(|constraint| match constraint {
                    Constraint::SumEquality {
                        sumands, result, ..
                    } => {
                        let mut sum = 0.0;
                        for &idx in sumands {
                            sum += system.values[idx];
                        }
                        let res = system.values[*result];
                        sum - res
                    }
                    Constraint::ProductEquality {
                        factors, result, ..
                    } => {
                        let mut prod = 1.0;
                        for &idx in factors {
                            prod *= system.values[idx];
                        }
                        let res = system.values[*result];
                        prod - res
                    }
                    Constraint::Range {
                        variable, min, max, ..
                    } => {
                        let val = system.values[*variable];
                        if val < *min {
                            val - *min
                        } else if val > *max {
                            val - *max
                        } else {
                            0.0
                        }
                    }
                    Constraint::DirectEquality { var_a, var_b, .. } => {
                        let val_a = system.values[*var_a];
                        let val_b = system.values[*var_b];
                        val_a - val_b
                    }
                })
                .collect()
        } else {
            system
                .constraints
                .iter()
                .map(|constraint| match constraint {
                    Constraint::SumEquality {
                        sumands, result, ..
                    } => {
                        let mut sum = 0.0;
                        for &idx in sumands {
                            sum += system.values[idx];
                        }
                        let res = system.values[*result];
                        sum - res
                    }
                    Constraint::ProductEquality {
                        factors, result, ..
                    } => {
                        let mut prod = 1.0;
                        for &idx in factors {
                            prod *= system.values[idx];
                        }
                        let res = system.values[*result];
                        prod - res
                    }
                    Constraint::Range {
                        variable, min, max, ..
                    } => {
                        let val = system.values[*variable];
                        if val < *min {
                            val - *min
                        } else if val > *max {
                            val - *max
                        } else {
                            0.0
                        }
                    }
                    Constraint::DirectEquality { var_a, var_b, .. } => {
                        let val_a = system.values[*var_a];
                        let val_b = system.values[*var_b];
                        val_a - val_b
                    }
                })
                .collect()
        };

        // Calculate total elastic energy
        let total_energy = if use_parallel {
            tensions.par_iter().map(|&t| t * t).sum()
        } else {
            tensions.iter().map(|&t| t * t).sum()
        };

        // Pass 2: Calculate the energy gradient for each variable (GATHER model)
        let gradient: Vec<f64> = if use_parallel {
            (0..system.values.len())
                .into_par_iter()
                .map(|var_idx| {
                    if system.is_fixed(var_idx) {
                        return 0.0;
                    }

                    let mut grad_i = 0.0;
                    if let Some(rest_indices) = system.variables_to_constraints.get(var_idx) {
                        for &r_idx in rest_indices {
                            let tension = tensions[r_idx];
                            let rest = &system.constraints[r_idx];
                            let deriv = rest.partial_derivative(var_idx, &system.values);
                            grad_i += 2.0 * tension * deriv;
                        }
                    }
                    grad_i
                })
                .collect()
        } else {
            (0..system.values.len())
                .map(|var_idx| {
                    if system.is_fixed(var_idx) {
                        return 0.0;
                    }

                    let mut grad_i = 0.0;
                    if let Some(rest_indices) = system.variables_to_constraints.get(var_idx) {
                        for &r_idx in rest_indices {
                            let tension = tensions[r_idx];
                            let rest = &system.constraints[r_idx];
                            let deriv = rest.partial_derivative(var_idx, &system.values);
                            grad_i += 2.0 * tension * deriv;
                        }
                    }
                    grad_i
                })
                .collect()
        };

        StressField {
            total_energy,
            tensions,
            gradient,
        }
    }
}
