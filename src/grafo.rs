use std::collections::HashMap;

/// Represents the logical and mathematical constraints governing the flat system.
/// Constraints use `usize` indices instead of string names to avoid slow lookups in hash maps.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Weighted sum: sumands[0] + sumands[1] + ... = result
    SumEquality {
        /// Identifier name of the constraint.
        name: String,
        /// Indices of the variables to sum.
        sumands: Vec<usize>,
        /// Index of the variable representing the sum result.
        result: usize,
    },
    /// Constant product (AMM curves): factors[0] * factors[1] * ... = result
    ProductEquality {
        /// Identifier name of the constraint.
        name: String,
        /// Indices of the variables to multiply.
        factors: Vec<usize>,
        /// Index of the variable representing the product result.
        result: usize,
    },
    /// Boundary limit: variable in range [min, max]
    Range {
        /// Identifier name of the constraint.
        name: String,
        /// Index of the variable constrained.
        variable: usize,
        /// Minimum allowed value.
        min: f64,
        /// Maximum allowed value.
        max: f64,
    },
    /// Direct equivalence: var_a = var_b
    DirectEquality {
        /// Identifier name of the constraint.
        name: String,
        /// Index of the first variable.
        var_a: usize,
        /// Index of the second variable.
        var_b: usize,
    },
}

impl Constraint {
    /// Returns the identifier name of the constraint.
    pub fn name(&self) -> &str {
        match self {
            Constraint::SumEquality { name, .. } => name,
            Constraint::ProductEquality { name, .. } => name,
            Constraint::Range { name, .. } => name,
            Constraint::DirectEquality { name, .. } => name,
        }
    }

    /// Calculates the partial derivative of the constraint stress with respect to variable `var_idx`.
    /// This method encapsulates local derivative behavior for lock-free parallelization.
    pub fn partial_derivative(&self, var_idx: usize, values: &[f64]) -> f64 {
        match self {
            Constraint::SumEquality {
                sumands, result, ..
            } => {
                let mut d = 0.0;
                for &s in sumands {
                    if s == var_idx {
                        d += 1.0;
                    }
                }
                if *result == var_idx {
                    d -= 1.0;
                }
                d
            }
            Constraint::ProductEquality {
                factors, result, ..
            } => {
                let mut d = 0.0;
                for (i, &f) in factors.iter().enumerate() {
                    if f == var_idx {
                        let mut prod = 1.0;
                        for (k, &other_f) in factors.iter().enumerate() {
                            if i != k {
                                prod *= values[other_f];
                            }
                        }
                        d += prod;
                    }
                }
                if *result == var_idx {
                    d -= 1.0;
                }
                d
            }
            Constraint::Range {
                variable, min, max, ..
            } => {
                if *variable == var_idx {
                    let val = values[*variable];
                    if val < *min || val > *max { 1.0 } else { 0.0 }
                } else {
                    0.0
                }
            }
            Constraint::DirectEquality { var_a, var_b, .. } => {
                let mut d = 0.0;
                if *var_a == var_idx {
                    d += 1.0;
                }
                if *var_b == var_idx {
                    d -= 1.0;
                }
                d
            }
        }
    }
}

/// Flat and contiguous matrix constraint system.
/// All information is stored in contiguous arrays in memory,
/// ready to be traversed sequentially to allow auto-vectorization (SIMD).
#[derive(Debug, Clone, Default)]
pub struct ConstraintSystem {
    /// Names of all indexed variables.
    pub names: Vec<String>,
    /// Real-time numerical values of the variables.
    pub values: Vec<f64>,
    /// Elasticity coefficients (0.0 indicates rigid / fixed).
    pub elasticities: Vec<f64>,
    /// Auxiliary mapping of name -> index. Used ONLY during construction,
    /// never during the inner optimization loop.
    pub variable_indices: HashMap<String, usize>,
    /// List of all constraints governing the system.
    pub constraints: Vec<Constraint>,
    /// Adjacency mapping: for each variable index, the list of constraint indices referencing it.
    /// Critical for lock-free parallel gather operations.
    pub variables_to_constraints: Vec<Vec<usize>>,
}

impl ConstraintSystem {
    /// Creates a new empty instance of `ConstraintSystem`.
    pub fn new() -> Self {
        ConstraintSystem {
            names: Vec::new(),
            values: Vec::new(),
            elasticities: Vec::new(),
            variable_indices: HashMap::new(),
            constraints: Vec::new(),
            variables_to_constraints: Vec::new(),
        }
    }

    /// Adds a variable to the flat system and returns its unique index.
    /// If the variable already exists, updates its properties.
    pub fn add_variable(&mut self, name: &str, value: f64, elasticity: f64) -> usize {
        let elasticity_val = elasticity.max(0.0);
        if let Some(&idx) = self.variable_indices.get(name) {
            self.values[idx] = value;
            self.elasticities[idx] = elasticity_val;
            idx
        } else {
            let idx = self.values.len();
            self.names.push(name.to_string());
            self.values.push(value);
            self.elasticities.push(elasticity_val);
            self.variable_indices.insert(name.to_string(), idx);
            idx
        }
    }

    /// Gets the index of a variable. If it does not exist, creates it with default values.
    pub fn get_or_create_variable(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.variable_indices.get(name) {
            idx
        } else {
            self.add_variable(name, 0.0, 1.0)
        }
    }

    /// Adds a constraint to the flat graph.
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Precomputes the adjacency mapping of variables to constraints.
    /// Must be called after adding all variables and constraints,
    /// and before running the solver loop.
    pub fn precompute_adjacencies(&mut self) {
        let num_vars = self.values.len();
        let mut mapping = vec![Vec::new(); num_vars];

        for (r_idx, rest) in self.constraints.iter().enumerate() {
            match rest {
                Constraint::SumEquality {
                    sumands, result, ..
                } => {
                    for &s in sumands {
                        mapping[s].push(r_idx);
                    }
                    mapping[*result].push(r_idx);
                }
                Constraint::ProductEquality {
                    factors, result, ..
                } => {
                    for &f in factors {
                        mapping[f].push(r_idx);
                    }
                    mapping[*result].push(r_idx);
                }
                Constraint::Range { variable, .. } => {
                    mapping[*variable].push(r_idx);
                }
                Constraint::DirectEquality { var_a, var_b, .. } => {
                    mapping[*var_a].push(r_idx);
                    mapping[*var_b].push(r_idx);
                }
            }
        }

        // Remove duplicates to avoid redundant calculations if a variable
        // appears multiple times in the same constraint
        for list in &mut mapping {
            list.sort_unstable();
            list.dedup();
        }

        self.variables_to_constraints = mapping;
    }

    /// Gets the current value of a variable directly by its index.
    #[inline(always)]
    pub fn get_value(&self, idx: usize) -> f64 {
        self.values[idx]
    }

    /// Updates the value of a variable directly.
    #[inline(always)]
    pub fn update_value(&mut self, idx: usize, new_value: f64) {
        self.values[idx] = new_value;
    }

    /// Checks if the variable at a given index is fixed (rigid).
    #[inline(always)]
    pub fn is_fixed(&self, idx: usize) -> bool {
        self.elasticities[idx] <= f64::EPSILON
    }

    /// Returns a map of names to current values (useful to gather results).
    pub fn map_values(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for (idx, name) in self.names.iter().enumerate() {
            map.insert(name.clone(), self.values[idx]);
        }
        map
    }
}
