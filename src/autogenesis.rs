use crate::grafo::{Constraint, ConstraintSystem};

/// Autogenesis module takes raw data flows and logical rules
/// in text format, parses them autonomously, and compiles the
/// flat contiguous memory representation of the constraint system.
pub struct Autogenesis;

impl Autogenesis {
    /// Compiles a text stream of structured variables and rules into the flat constraint system.
    ///
    /// # Supported syntax:
    /// - `fixed(A, 10)` -> Rigid variable A with value 10.0
    /// - `var(X, 12)` -> Elastic variable X with value 12.0 and elasticity 1.0 (default)
    /// - `elastic(Z, 5, 2.5)` -> Elastic variable Z with value 5.0 and elasticity 2.5
    /// - `range(X, 0, 100)` -> Range constraint: X in [0.0, 100.0]
    /// - `A + B = C` -> Sum equality constraint: A + B = C
    /// - `A * B = C` -> Product equality constraint: A * B = C
    /// - `A = B` -> Direct equality constraint: A = B
    pub fn compile_raw_flow(data: &str) -> Result<ConstraintSystem, String> {
        let mut system = ConstraintSystem::new();
        let mut constraint_count = 0;

        for (line_num, line) in data.lines().enumerate() {
            let line = line.trim();

            // Ignore empty lines, comments, and decorative marks
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            if line.starts_with("fixed(") && line.ends_with(')') {
                // fixed(name, value)
                let inner = &line[6..line.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Line {}: fixed requires exactly 2 parameters",
                        line_num + 1
                    ));
                }
                let name = parts[0];
                let value = parts[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Line {}: invalid value for fixed variable: {}",
                        line_num + 1,
                        e
                    )
                })?;
                system.add_variable(name, value, 0.0);
            } else if line.starts_with("elastic(") && line.ends_with(')') {
                // elastic(name, value, elasticity)
                let inner = &line[8..line.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 3 {
                    return Err(format!(
                        "Line {}: elastic requires exactly 3 parameters",
                        line_num + 1
                    ));
                }
                let name = parts[0];
                let value = parts[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Line {}: invalid value for elastic variable: {}",
                        line_num + 1,
                        e
                    )
                })?;
                let elasticity = parts[2]
                    .parse::<f64>()
                    .map_err(|e| format!("Line {}: invalid elasticity: {}", line_num + 1, e))?;
                system.add_variable(name, value, elasticity);
            } else if line.starts_with("var(") && line.ends_with(')') {
                // var(name, value)
                let inner = &line[4..line.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Line {}: var requires exactly 2 parameters",
                        line_num + 1
                    ));
                }
                let name = parts[0];
                let value = parts[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Line {}: invalid value for standard variable: {}",
                        line_num + 1,
                        e
                    )
                })?;
                system.add_variable(name, value, 1.0);
            } else if line.starts_with("range(") && line.ends_with(')') {
                // range(variable, min, max)
                let inner = &line[6..line.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 3 {
                    return Err(format!(
                        "Line {}: range requires exactly 3 parameters",
                        line_num + 1
                    ));
                }
                let var_name = parts[0];
                let min = parts[1].parse::<f64>().map_err(|e| {
                    format!("Line {}: invalid minimum range value: {}", line_num + 1, e)
                })?;
                let max = parts[2].parse::<f64>().map_err(|e| {
                    format!("Line {}: invalid maximum range value: {}", line_num + 1, e)
                })?;

                let var_idx = system.get_or_create_variable(var_name);
                constraint_count += 1;

                system.add_constraint(Constraint::Range {
                    name: format!("autogen_range_{}", constraint_count),
                    variable: var_idx,
                    min,
                    max,
                });
            } else if line.contains('=') {
                // Equation: Left = Right
                let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Line {}: invalid equality format (must contain exactly one '=')",
                        line_num + 1
                    ));
                }
                let left = parts[0];
                let right = parts[1];

                // The right side commonly acts as the result variable
                let result_idx = system.get_or_create_variable(right);

                if left.contains('+') {
                    // Sumands separated by '+'
                    let sumands_strs: Vec<&str> = left.split('+').map(|s| s.trim()).collect();
                    let sumands_indices: Vec<usize> = sumands_strs
                        .iter()
                        .map(|&s| system.get_or_create_variable(s))
                        .collect();

                    constraint_count += 1;
                    system.add_constraint(Constraint::SumEquality {
                        name: format!("autogen_sum_{}", constraint_count),
                        sumands: sumands_indices,
                        result: result_idx,
                    });
                } else if left.contains('*') {
                    // Factors separated by '*'
                    let factors_strs: Vec<&str> = left.split('*').map(|s| s.trim()).collect();
                    let factors_indices: Vec<usize> = factors_strs
                        .iter()
                        .map(|&s| system.get_or_create_variable(s))
                        .collect();

                    constraint_count += 1;
                    system.add_constraint(Constraint::ProductEquality {
                        name: format!("autogen_product_{}", constraint_count),
                        factors: factors_indices,
                        result: result_idx,
                    });
                } else {
                    // Direct equality: A = B
                    let var_a_idx = system.get_or_create_variable(left);
                    constraint_count += 1;
                    system.add_constraint(Constraint::DirectEquality {
                        name: format!("autogen_direct_{}", constraint_count),
                        var_a: var_a_idx,
                        var_b: result_idx,
                    });
                }
            } else {
                return Err(format!(
                    "Line {}: unrecognized instruction or invalid syntax: '{}'",
                    line_num + 1,
                    line
                ));
            }
        }

        system.precompute_adjacencies();
        Ok(system)
    }
}
