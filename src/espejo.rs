use crate::estres::CampoEstres;
use crate::grafo::{Restriccion, SistemaRestricciones};
use std::collections::HashMap;

/// Los estados de la solución devuelta por el resolvedor elástico de S.P.E.C.U.L.A.M.
#[derive(Debug, Clone)]
pub enum SolucionEspejo {
    /// El sistema es completamente coherente y no tiene tensión inicial significativa.
    Directa {
        /// Mapa de nombres de variables a sus valores finales.
        valores: HashMap<String, f64>,
    },
    /// El sistema contenía contradicciones o tensiones. Se proponen ajustes elásticos
    /// en las variables o colapsos de fase en las restricciones rígidas para equilibrarlo.
    Pista {
        /// Mapa de nombres de variables a sus valores de entrada originales.
        valores_originales: HashMap<String, f64>,
        /// Mapa de nombres de variables a sus nuevos valores equilibrados.
        valores_ajustados: HashMap<String, f64>,
        /// Mapa con las desviaciones aplicadas a las variables maleables.
        desviaciones: HashMap<String, f64>,
        /// Mapa de restricciones rígidas con su tensión residual calculada.
        tensiones_residuales: HashMap<String, f64>,
        /// Explicación heurística y legible de la solución o de las inconsistencias.
        explicacion: String,
    },
    /// La complejidad o las contradicciones son tan altas que no se pudo encontrar un mínimo
    /// local estable dentro del límite de pasos.
    ComplejidadAlta {
        /// Campo de estrés final del resolvedor.
        estres: CampoEstres,
        /// Mensaje descriptivo con el motivo de la complejidad/fallo.
        mensaje: String,
    },
}

/// Motor de optimización elástica que disipa el estrés local mediante descenso de gradiente.
pub struct MotorSpeculam {
    /// Tasa de aprendizaje/paso de optimización para el descenso de gradiente.
    pub learning_rate: f64,
    /// Número máximo de iteraciones permitidas para el resolvedor.
    pub max_pasos: usize,
    /// Tolerancia de energía residual por debajo de la cual el sistema se considera resuelto.
    pub tolerancia: f64,
    /// Presupuesto estricto de tiempo de ejecución en microsegundos (opcional).
    pub max_duracion_microsegundos: Option<u64>,
}

impl Default for MotorSpeculam {
    fn default() -> Self {
        MotorSpeculam {
            learning_rate: 0.01,
            max_pasos: 1000,
            tolerancia: 1e-10, // Tolerancia fina para equilibrar tensiones residuales
            max_duracion_microsegundos: None, // Sin límite de tiempo por defecto para evitar timeouts en tests/debug
        }
    }
}

impl MotorSpeculam {
    /// Crea una nueva instancia de `MotorSpeculam` con los parámetros por defecto.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evalúa el sistema plano y busca rutas alternativas mediante relajación elástica de memoria contigua.
    pub fn evaluar(&self, sistema: &SistemaRestricciones) -> SolucionEspejo {
        let estres_inicial = CampoEstres::calcular(sistema);

        // Si la energía inicial es casi cero, el sistema ya es coherente.
        if estres_inicial.energia_total < self.tolerancia || estres_inicial.energia_total.is_nan() {
            return SolucionEspejo::Directa {
                valores: sistema.mapear_valores(),
            };
        }

        let mut sistema_trabajo = sistema.clone();
        let mut mejor_energia = estres_inicial.energia_total;
        let mut mejor_valores = sistema_trabajo.valores.clone();
        let mut pasos_sin_mejora = 0;
        let t_inicio = std::time::Instant::now();

        // Bucle de optimización ultra rápido por indexación directa de memoria
        for _ in 0..self.max_pasos {
            // Control de tiempo en tiempo real (microsegundos)
            if self
                .max_duracion_microsegundos
                .is_some_and(|limite| t_inicio.elapsed().as_micros() as u64 >= limite)
            {
                break;
            }

            let estres_actual = CampoEstres::calcular(&sistema_trabajo);

            // Defensa contra NaN / Infinity en la energía
            if estres_actual.energia_total.is_nan() || estres_actual.energia_total.is_infinite() {
                break;
            }

            if estres_actual.energia_total < self.tolerancia {
                mejor_valores = sistema_trabajo.valores.clone();
                break;
            }

            if estres_actual.energia_total < mejor_energia {
                mejor_energia = estres_actual.energia_total;
                mejor_valores = sistema_trabajo.valores.clone();
                pasos_sin_mejora = 0;
            } else {
                pasos_sin_mejora += 1;
                if pasos_sin_mejora > 50 {
                    break;
                }
            }

            // Aplicar el gradiente sobre las variables maleables con recorte de gradiente por componente (damping)
            // para evitar que variables con escalas masivas (como productos de AMM) congelen el resto del sistema.
            let len = sistema_trabajo.valores.len();
            let mut detected_nan = false;

            for idx in 0..len {
                if !sistema_trabajo.es_fija(idx) {
                    let grad_val = estres_actual.gradiente[idx];

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
                    let delta =
                        -self.learning_rate * sistema_trabajo.elasticidades[idx] * clipped_g;
                    sistema_trabajo.valores[idx] += delta;
                }
            }

            if detected_nan {
                break;
            }
        }

        // Restaurar el mejor estado estable encontrado (con menor energía)
        sistema_trabajo.valores = mejor_valores;
        let estres_final = CampoEstres::calcular(&sistema_trabajo);

        let valores_originales = sistema.mapear_valores();
        let valores_ajustados = sistema_trabajo.mapear_valores();

        let mut desviaciones = HashMap::new();
        for (k, v_orig) in &valores_originales {
            let v_ajust = valores_ajustados.get(k).unwrap_or(v_orig);
            let diff = v_ajust - v_orig;
            if diff.abs() > 1e-5 {
                desviaciones.insert(k.clone(), diff);
            }
        }

        let mut tensiones_residuales = HashMap::new();
        for (r_idx, &tension) in estres_final.tensiones.iter().enumerate() {
            if tension.abs() > 1e-5 {
                let nombre_r = sistema.restricciones[r_idx].nombre().to_string();
                tensiones_residuales.insert(nombre_r, tension);
            }
        }

        // Construir la explicación heurística a partir de los datos mapeados
        let mut explicacion = String::new();
        explicacion.push_str("--- ANÁLISIS DE ESPEJO LÓGICO PLANO (S.P.E.C.U.L.A.M. v2) ---\n");

        if !desviaciones.is_empty() {
            explicacion.push_str("• Relajaciones elásticas aplicadas en variables maleables:\n");
            for (var, delta) in &desviaciones {
                let orig = valores_originales.get(var).unwrap();
                let nuevo = valores_ajustados.get(var).unwrap();
                explicacion.push_str(&format!(
                    "  - Variable '{}': {:.4} -> {:.4} (desviación de [{:+.4}])\n",
                    var, orig, nuevo, delta
                ));
            }
        }

        if !tensiones_residuales.is_empty() {
            explicacion.push_str(
                "• Pistas estructurales detectadas para resolver contradicciones rígidas:\n",
            );
            for (restriccion_nombre, tension) in &tensiones_residuales {
                if let Some(rest) = sistema
                    .restricciones
                    .iter()
                    .find(|r| r.nombre() == restriccion_nombre)
                {
                    match rest {
                        Restriccion::IgualdadSuma {
                            sumandos,
                            resultado,
                            ..
                        } => {
                            let nombres_sumandos: Vec<String> = sumandos
                                .iter()
                                .map(|&idx| sistema.nombres[idx].clone())
                                .collect();
                            let nombre_res = &sistema.nombres[*resultado];

                            let suma_real: f64 = sumandos
                                .iter()
                                .map(|&idx| sistema_trabajo.valores[idx])
                                .sum();
                            let res_esperado = sistema_trabajo.valores[*resultado];

                            explicacion.push_str(&format!(
                                "  - Relación '{}': ({} = {}) falló. La suma real es {}, pero se esperaba {}.\n",
                                restriccion_nombre, nombres_sumandos.join(" + "), nombre_res, suma_real, res_esperado
                            ));
                            explicacion.push_str(&format!(
                                "    >>> COLAPSO DE FASE: Para equilibrar la ecuación, se requiere un ajuste de [{:+.4}] en la suma.\n",
                                -tension
                            ));
                            explicacion.push_str(&format!(
                                "    >>> RUTA OCULTA: ({}) {:+.4} = {}\n",
                                sumandos
                                    .iter()
                                    .map(|&idx| format!("{}", sistema_trabajo.valores[idx]))
                                    .collect::<Vec<_>>()
                                    .join(" + "),
                                -tension,
                                res_esperado
                            ));
                        }
                        Restriccion::IgualdadProducto {
                            factores,
                            resultado,
                            ..
                        } => {
                            let nombres_factores: Vec<String> = factores
                                .iter()
                                .map(|&idx| sistema.nombres[idx].clone())
                                .collect();
                            let nombre_res = &sistema.nombres[*resultado];

                            let prod_real: f64 = factores
                                .iter()
                                .map(|&idx| sistema_trabajo.valores[idx])
                                .product();
                            let res_esperado = sistema_trabajo.valores[*resultado];

                            explicacion.push_str(&format!(
                                "  - Relación '{}': ({} = {}) falló. El producto real es {}, pero se esperaba {}.\n",
                                restriccion_nombre, nombres_factores.join(" * "), nombre_res, prod_real, res_esperado
                            ));
                            explicacion.push_str(&format!(
                                "    >>> COLAPSO DE FASE: Para equilibrar la ecuación, se requiere un ajuste de [{:+.4}] en el producto.\n",
                                -tension
                            ));
                        }
                        Restriccion::Rango {
                            variable, min, max, ..
                        } => {
                            let nombre_var = &sistema.nombres[*variable];
                            let val = sistema_trabajo.valores[*variable];
                            explicacion.push_str(&format!(
                                "  - Rango '{}': '{}' con valor {} está fuera del intervalo [{}, {}].\n",
                                restriccion_nombre, nombre_var, val, min, max
                            ));
                            explicacion.push_str(&format!(
                                "    >>> COLAPSO DE FASE: Mover '{}' por [{:+.4}] para entrar en el rango permitido.\n",
                                nombre_var, -tension
                            ));
                        }
                        Restriccion::IgualdadDirecta { var_a, var_b, .. } => {
                            let nombre_a = &sistema.nombres[*var_a];
                            let nombre_b = &sistema.nombres[*var_b];
                            let val_a = sistema_trabajo.valores[*var_a];
                            let val_b = sistema_trabajo.valores[*var_b];
                            explicacion.push_str(&format!(
                                "  - Igualdad '{}': '{}' ({}) != '{}' ({}).\n",
                                restriccion_nombre, nombre_a, val_a, nombre_b, val_b
                            ));
                            explicacion.push_str(&format!(
                                "    >>> COLAPSO DE FASE: Forzar colapso de simetría con una desviación de [{:+.4}].\n",
                                -tension
                            ));
                        }
                    }
                }
            }
        }

        let divergiendo = estres_final.energia_total.is_nan()
            || estres_final.energia_total.is_infinite()
            || estres_final.energia_total > estres_inicial.energia_total * 2.0;

        if divergiendo {
            SolucionEspejo::ComplejidadAlta {
                estres: estres_final,
                mensaje:
                    "El campo de estrés divergió. Reducción inestable o desbordamiento numérico."
                        .to_string(),
            }
        } else {
            SolucionEspejo::Pista {
                valores_originales,
                valores_ajustados,
                desviaciones,
                tensiones_residuales,
                explicacion,
            }
        }
    }
}
