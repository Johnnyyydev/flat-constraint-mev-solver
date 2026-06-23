use crate::estres::StressField;
use crate::grafo::ConstraintSystem;
use std::collections::HashMap;

/// Structure responsible for continuous-discrete hybrid optimization
/// and collapsing variables to integer coordinates via topological annealing.
pub struct QuantumJumper {
    /// Learning rate step for gradient descent.
    pub learning_rate: f64,
    /// Maximum iterations allowed per annealing cycle.
    pub max_steps: usize,
    /// Residual energy tolerance below which a discrete solution is accepted.
    pub tolerance: f64,
    /// Maximum strength of the periodic crystallization potential (K_int).
    pub crystallization_strength: f64,
}

impl Default for QuantumJumper {
    fn default() -> Self {
        QuantumJumper {
            learning_rate: 0.01,
            max_steps: 1000,
            tolerance: 1e-6,
            crystallization_strength: 8.0,
        }
    }
}

impl QuantumJumper {
    /// Creates a new instance of `QuantumJumper` with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Tries to find a combination of integer values for the given discrete variables
    /// that minimizes or completely eliminates system stress.
    ///
    /// This method uses a physics-inspired approach, injecting a periodic
    /// sinusoidal potential that forces the specified variables to crystallize into integer values
    /// as the thermal gradient descent progresses.
    pub fn jump_discrete_space(
        &self,
        system: &ConstraintSystem,
        discrete_vars: &[usize],
    ) -> Option<HashMap<String, f64>> {
        let mut working_system = system.clone();

        // Ultra fast Linear Congruential Generator (LCG) to avoid external dependency on rand crates
        let mut lcg_seed = 987654321u32;
        let mut next_random = || {
            lcg_seed = lcg_seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (lcg_seed as f64) / (u32::MAX as f64)
        };

        let mut best_discrete_energy = f64::MAX;
        let mut best_discrete_config: Option<Vec<f64>> = None;

        // Try multiple cycles of annealing/crystallization with thermal impulses (Quantum Kicks)
        for cycle in 0..5 {
            if cycle > 0 {
                // Apply a "Quantum Kick" (thermal impulse) to jump to another region of the topological space
                for idx in 0..working_system.values.len() {
                    if !working_system.is_fixed(idx) {
                        // Fluctuation proportional to current cycle distance
                        let kick = (next_random() - 0.5) * 5.0 / (cycle as f64);
                        working_system.values[idx] += kick;
                    }
                }
            }

            for step in 0..self.max_steps {
                let mut current_stress = StressField::calculate(&working_system);

                // Exponential crystallization strength ramp (K_int)
                let progress = step as f64 / self.max_steps as f64;
                let k_int = self.crystallization_strength * progress.powi(2);

                // Inject periodic crystallization gradient E_int = K * sin^2(pi * x)
                // dE_int/dx = K * pi * sin(2 * pi * x)
                for &idx in discrete_vars {
                    let val = working_system.values[idx];
                    let grad_int =
                        k_int * std::f64::consts::PI * (2.0 * std::f64::consts::PI * val).sin();
                    current_stress.gradient[idx] += grad_int;
                }

                // Gradient Damping/Clipping
                let mut gradient_norm = 0.0;
                for &g in &current_stress.gradient {
                    gradient_norm += g * g;
                }
                let gradient_norm = gradient_norm.sqrt();

                let clipping_factor = if gradient_norm > 15.0 {
                    15.0 / gradient_norm
                } else {
                    1.0
                };

                // Update elastic variables
                let len = working_system.values.len();
                for idx in 0..len {
                    if !working_system.is_fixed(idx) {
                        let grad_val = current_stress.gradient[idx] * clipping_factor;
                        let delta =
                            -self.learning_rate * working_system.elasticities[idx] * grad_val;
                        working_system.values[idx] += delta;
                    }
                }

                // Every 20 steps, perform an experimental discrete round-off (wavefunction collapse)
                if step % 20 == 0 || step == self.max_steps - 1 {
                    let mut collapsed_system = working_system.clone();
                    for &idx in discrete_vars {
                        collapsed_system.values[idx] = working_system.values[idx].round();
                    }

                    let collapsed_stress = StressField::calculate(&collapsed_system);

                    // If collapse dissipates stress below tolerance, we have found it
                    if collapsed_stress.total_energy < self.tolerance {
                        return Some(collapsed_system.map_values());
                    }

                    // Record the best discrete collapse if energy is lower
                    if collapsed_stress.total_energy < best_discrete_energy {
                        best_discrete_energy = collapsed_stress.total_energy;
                        best_discrete_config = Some(collapsed_system.values.clone());
                    }
                }
            }
        }

        // If no perfect collapse found (energy < tolerance), return the best config
        // obtained, as long as it has reasonable energy.
        if let Some(final_values) = best_discrete_config.filter(|_| best_discrete_energy < 1.0) {
            let mut final_system = working_system.clone();
            final_system.values = final_values;
            return Some(final_system.map_values());
        }

        None
    }
}
