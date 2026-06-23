use crate::grafo::{Restriccion, SistemaRestricciones};

/// El módulo de Autogénesis toma flujos crudos de datos y reglas lógicas
/// en formato texto, los parsea de manera autónoma y compila la
/// representación matricial plana del sistema de restricciones.
pub struct Autogenesis;

impl Autogenesis {
    /// Compila un texto con variables y reglas estructuradas en el sistema plano.
    ///
    /// # Sintaxis soportada:
    /// - `fixed(A, 10)` -> Variable rígida/fija A con valor 10.0
    /// - `var(X, 12)` -> Variable maleable X con valor 12.0 y elasticidad 1.0 (defecto)
    /// - `elastic(Z, 5, 2.5)` -> Variable Z con valor 5.0 y elasticidad 2.5
    /// - `rango(X, 0, 100)` -> Restricción de rango: X en [0.0, 100.0]
    /// - `A + B = C` -> Restricción de suma: A + B = C
    /// - `A * B = C` -> Restricción de multiplicación: A * B = C
    /// - `A = B` -> Restricción de igualdad directa: A = B
    pub fn compilar_flujo_crudo(datos: &str) -> Result<SistemaRestricciones, String> {
        let mut sistema = SistemaRestricciones::new();
        let mut contador_restricciones = 0;

        for (num_linea, linea) in datos.lines().enumerate() {
            let linea = linea.trim();

            // Ignorar líneas vacías, comentarios y marcas decorativas
            if linea.is_empty() || linea.starts_with('#') || linea.starts_with("//") {
                continue;
            }

            if linea.starts_with("fixed(") && linea.ends_with(')') {
                // fixed(nombre, valor)
                let interior = &linea[6..linea.len() - 1];
                let partes: Vec<&str> = interior.split(',').map(|s| s.trim()).collect();
                if partes.len() != 2 {
                    return Err(format!(
                        "Línea {}: fixed requiere exactamente 2 parámetros",
                        num_linea + 1
                    ));
                }
                let nombre = partes[0];
                let valor = partes[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Línea {}: valor inválido para variable rígida: {}",
                        num_linea + 1,
                        e
                    )
                })?;
                sistema.agregar_variable(nombre, valor, 0.0);
            } else if linea.starts_with("elastic(") && linea.ends_with(')') {
                // elastic(nombre, valor, elasticidad)
                let interior = &linea[8..linea.len() - 1];
                let partes: Vec<&str> = interior.split(',').map(|s| s.trim()).collect();
                if partes.len() != 3 {
                    return Err(format!(
                        "Línea {}: elastic requiere exactamente 3 parámetros",
                        num_linea + 1
                    ));
                }
                let nombre = partes[0];
                let valor = partes[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Línea {}: valor inválido para variable elástica: {}",
                        num_linea + 1,
                        e
                    )
                })?;
                let elasticidad = partes[2]
                    .parse::<f64>()
                    .map_err(|e| format!("Línea {}: elasticidad inválida: {}", num_linea + 1, e))?;
                sistema.agregar_variable(nombre, valor, elasticidad);
            } else if linea.starts_with("var(") && linea.ends_with(')') {
                // var(nombre, valor)
                let interior = &linea[4..linea.len() - 1];
                let partes: Vec<&str> = interior.split(',').map(|s| s.trim()).collect();
                if partes.len() != 2 {
                    return Err(format!(
                        "Línea {}: var requiere exactamente 2 parámetros",
                        num_linea + 1
                    ));
                }
                let nombre = partes[0];
                let valor = partes[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Línea {}: valor de variable estándar inválido: {}",
                        num_linea + 1,
                        e
                    )
                })?;
                sistema.agregar_variable(nombre, valor, 1.0);
            } else if linea.starts_with("rango(") && linea.ends_with(')') {
                // rango(variable, min, max)
                let interior = &linea[6..linea.len() - 1];
                let partes: Vec<&str> = interior.split(',').map(|s| s.trim()).collect();
                if partes.len() != 3 {
                    return Err(format!(
                        "Línea {}: rango requiere exactamente 3 parámetros",
                        num_linea + 1
                    ));
                }
                let var_nombre = partes[0];
                let min = partes[1].parse::<f64>().map_err(|e| {
                    format!(
                        "Línea {}: valor min de rango inválido: {}",
                        num_linea + 1,
                        e
                    )
                })?;
                let max = partes[2].parse::<f64>().map_err(|e| {
                    format!(
                        "Línea {}: valor max de rango inválido: {}",
                        num_linea + 1,
                        e
                    )
                })?;

                let var_idx = sistema.obtener_o_crear_variable(var_nombre);
                contador_restricciones += 1;

                sistema.agregar_restriccion(Restriccion::Rango {
                    nombre: format!("autogen_rango_{}", contador_restricciones),
                    variable: var_idx,
                    min,
                    max,
                });
            } else if linea.contains('=') {
                // Ecuación: Izquierda = Derecha
                let partes: Vec<&str> = linea.split('=').map(|s| s.trim()).collect();
                if partes.len() != 2 {
                    return Err(format!(
                        "Línea {}: formato de igualdad inválido (debe contener un solo '=')",
                        num_linea + 1
                    ));
                }
                let izquierda = partes[0];
                let derecha = partes[1];

                // El lado derecho actúa comúnmente como la variable resultado
                let resultado_idx = sistema.obtener_o_crear_variable(derecha);

                if izquierda.contains('+') {
                    // Sumandos separados por '+'
                    let sumandos_strs: Vec<&str> = izquierda.split('+').map(|s| s.trim()).collect();
                    let sumandos_indices: Vec<usize> = sumandos_strs
                        .iter()
                        .map(|&s| sistema.obtener_o_crear_variable(s))
                        .collect();

                    contador_restricciones += 1;
                    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
                        nombre: format!("autogen_suma_{}", contador_restricciones),
                        sumandos: sumandos_indices,
                        resultado: resultado_idx,
                    });
                } else if izquierda.contains('*') {
                    // Factores separados por '*'
                    let factores_strs: Vec<&str> = izquierda.split('*').map(|s| s.trim()).collect();
                    let factores_indices: Vec<usize> = factores_strs
                        .iter()
                        .map(|&s| sistema.obtener_o_crear_variable(s))
                        .collect();

                    contador_restricciones += 1;
                    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
                        nombre: format!("autogen_producto_{}", contador_restricciones),
                        factores: factores_indices,
                        resultado: resultado_idx,
                    });
                } else {
                    // Igualdad directa: A = B
                    let var_a_idx = sistema.obtener_o_crear_variable(izquierda);
                    contador_restricciones += 1;
                    sistema.agregar_restriccion(Restriccion::IgualdadDirecta {
                        nombre: format!("autogen_directa_{}", contador_restricciones),
                        var_a: var_a_idx,
                        var_b: resultado_idx,
                    });
                }
            } else {
                return Err(format!(
                    "Línea {}: instrucción no reconocida o sintaxis inválida: '{}'",
                    num_linea + 1,
                    linea
                ));
            }
        }

        sistema.precalcular_adyacencias();
        Ok(sistema)
    }
}
