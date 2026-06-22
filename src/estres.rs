use rayon::prelude::*;
use crate::grafo::{SistemaRestricciones, Restriccion};

/// Contiene el estado del campo de estrés en un instante dado usando arrays contiguos sin alocación.
#[derive(Debug, Clone)]
pub struct CampoEstres {
    /// Energía elástica total almacenada en el sistema (suma de tensiones al cuadrado).
    pub energia_total: f64,
    /// Vector plano de tensiones alineado con los índices de `sistema.restricciones`.
    /// Al usar Vec<f64> en lugar de HashMap<String, f64>, eliminamos el 100% de las alocaciones
    /// de memoria en el hot path, desbloqueando el verdadero rendimiento SIMD y Rayon.
    pub tensiones: Vec<f64>,
    /// Gradiente de energía respecto a cada variable indexada directamente por su posición.
    pub gradiente: Vec<f64>,
}

impl CampoEstres {
    /// Calcula el campo de estrés actual del sistema de restricciones plano libre de alocaciones.
    pub fn calcular(sistema: &SistemaRestricciones) -> Self {
        // Pasada 1: Calcular la tensión de cada restricción en paralelo con Rayon (sin alocar Strings)
        let tensiones: Vec<f64> = sistema.restricciones
            .par_iter()
            .map(|restriccion| {
                match restriccion {
                    Restriccion::IgualdadSuma { sumandos, resultado, .. } => {
                        let mut suma = 0.0;
                        for &idx in sumandos {
                            suma += sistema.valores[idx];
                        }
                        let res = sistema.valores[*resultado];
                        suma - res
                    },
                    Restriccion::IgualdadProducto { factores, resultado, .. } => {
                        let mut prod = 1.0;
                        for &idx in factores {
                            prod *= sistema.valores[idx];
                        }
                        let res = sistema.valores[*resultado];
                        prod - res
                    },
                    Restriccion::Rango { variable, min, max, .. } => {
                        let val = sistema.valores[*variable];
                        if val < *min {
                            val - *min
                        } else if val > *max {
                            val - *max
                        } else {
                            0.0
                        }
                    },
                    Restriccion::IgualdadDirecta { var_a, var_b, .. } => {
                        let val_a = sistema.valores[*var_a];
                        let val_b = sistema.valores[*var_b];
                        val_a - val_b
                    }
                }
            })
            .collect();

        // Calcular la energía total elástica en paralelo usando sumatorio
        let energia_total = tensiones.par_iter().map(|&t| t * t).sum();

        // Pasada 2: Calcular el gradiente de energía para cada variable de forma paralela (modelo GATHER)
        let gradiente: Vec<f64> = (0..sistema.valores.len())
            .into_par_iter()
            .map(|var_idx| {
                if sistema.es_fija(var_idx) {
                    return 0.0;
                }

                let mut grad_i = 0.0;
                if let Some(rest_indices) = sistema.variables_a_restricciones.get(var_idx) {
                    for &r_idx in rest_indices {
                        let tension = tensiones[r_idx];
                        let rest = &sistema.restricciones[r_idx];
                        let deriv = rest.derivada_parcial(var_idx, &sistema.valores);
                        grad_i += 2.0 * tension * deriv;
                    }
                }
                grad_i
            })
            .collect();

        CampoEstres {
            energia_total,
            tensiones,
            gradiente,
        }
    }
}
