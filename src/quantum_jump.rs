use crate::estres::CampoEstres;
use crate::grafo::SistemaRestricciones;
use std::collections::HashMap;

/// Estructura encargada de realizar la optimización híbrida continuo-discreta
/// y colapsar variables a coordenadas enteras mediante recocido topológico.
pub struct QuantumJumper {
    /// Tasa de aprendizaje/paso de optimización para el descenso de gradiente.
    pub learning_rate: f64,
    /// Número máximo de iteraciones permitidas por ciclo de recocido.
    pub max_pasos: usize,
    /// Tolerancia de energía residual aceptada para considerar una solución discreta exitosa.
    pub tolerancia: f64,
    /// Fuerza máxima del potencial periódico de cristalización (K_int).
    pub fuerza_cristalizacion: f64,
}

impl Default for QuantumJumper {
    fn default() -> Self {
        QuantumJumper {
            learning_rate: 0.01,
            max_pasos: 1000,
            tolerancia: 1e-6,
            fuerza_cristalizacion: 8.0,
        }
    }
}

impl QuantumJumper {
    /// Crea una nueva instancia de `QuantumJumper` con la configuración por defecto.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intenta encontrar una combinación de valores enteros para las variables discretas dadas
    /// que minimice o anule por completo la tensión del sistema.
    ///
    /// Este método utiliza una técnica inspirada en la física cuántica, inyectando un potencial
    /// periódico sinusoidal que obliga a las variables especificadas a cristalizar en valores enteros
    /// a medida que avanza el descenso de gradiente térmico.
    pub fn saltar_espacio_discreto(
        &self,
        sistema: &SistemaRestricciones,
        var_discretas: &[usize],
    ) -> Option<HashMap<String, f64>> {
        let mut sistema_trabajo = sistema.clone();

        // Generador congruencial lineal (LCG) ultra simple y rápido para evitar dependencias de crates de azar
        let mut lcg_seed = 987654321u32;
        let mut next_random = || {
            lcg_seed = lcg_seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (lcg_seed as f64) / (u32::MAX as f64)
        };

        let mut mejor_energia_discreta = f64::MAX;
        let mut mejor_configuracion_discreta: Option<Vec<f64>> = None;

        // Intentamos múltiples ciclos de recocido/cristalización con impulsos térmicos (Quantum Kicks)
        for ciclo in 0..5 {
            if ciclo > 0 {
                // Aplicar un "Quantum Kick" (impulso térmico) para saltar a otra zona del espacio topológico
                for idx in 0..sistema_trabajo.valores.len() {
                    if !sistema_trabajo.es_fija(idx) {
                        // Flctuación proporcional a la distancia del ciclo
                        let kick = (next_random() - 0.5) * 5.0 / (ciclo as f64);
                        sistema_trabajo.valores[idx] += kick;
                    }
                }
            }

            for paso in 0..self.max_pasos {
                let mut estres_actual = CampoEstres::calcular(&sistema_trabajo);

                // Rampas exponenciales de fuerza de cristalización (K_int)
                let progreso = paso as f64 / self.max_pasos as f64;
                let k_int = self.fuerza_cristalizacion * progreso.powi(2);

                // Inyectar el gradiente periódico E_int = K * sin^2(pi * x)
                // dE_int/dx = K * pi * sin(2 * pi * x)
                for &idx in var_discretas {
                    let val = sistema_trabajo.valores[idx];
                    let grad_int =
                        k_int * std::f64::consts::PI * (2.0 * std::f64::consts::PI * val).sin();
                    estres_actual.gradiente[idx] += grad_int;
                }

                // Damping/Recorte de gradiente
                let mut norma_gradiente = 0.0;
                for &g in &estres_actual.gradiente {
                    norma_gradiente += g * g;
                }
                let norma_gradiente = norma_gradiente.sqrt();

                let factor_clipping = if norma_gradiente > 15.0 {
                    15.0 / norma_gradiente
                } else {
                    1.0
                };

                // Actualizar variables maleables
                let len = sistema_trabajo.valores.len();
                for idx in 0..len {
                    if !sistema_trabajo.es_fija(idx) {
                        let grad_val = estres_actual.gradiente[idx] * factor_clipping;
                        let delta =
                            -self.learning_rate * sistema_trabajo.elasticidades[idx] * grad_val;
                        sistema_trabajo.valores[idx] += delta;
                    }
                }

                // Cada 20 pasos, realizamos un "salto discreto experimental" (colapso de onda)
                // redondeando las variables a enteros y verificando si disipan el estrés elástico.
                if paso % 20 == 0 || paso == self.max_pasos - 1 {
                    let mut sistema_colapsado = sistema_trabajo.clone();
                    for &idx in var_discretas {
                        sistema_colapsado.valores[idx] = sistema_trabajo.valores[idx].round();
                    }

                    let estres_colapsado = CampoEstres::calcular(&sistema_colapsado);

                    // Si el colapso disipa la tensión por debajo de la tolerancia, lo hemos encontrado
                    if estres_colapsado.energia_total < self.tolerancia {
                        return Some(sistema_colapsado.mapear_valores());
                    }

                    // Registrar el mejor colapso discreto si la energía es menor
                    if estres_colapsado.energia_total < mejor_energia_discreta {
                        mejor_energia_discreta = estres_colapsado.energia_total;
                        mejor_configuracion_discreta = Some(sistema_colapsado.valores.clone());
                    }
                }
            }
        }

        // Si no encontramos un colapso perfecto (energía < tolerancia), retornamos el mejor
        // que hayamos obtenido en el proceso, siempre que la energía no sea astronómica.
        if let Some(valores_finales) =
            mejor_configuracion_discreta.filter(|_| mejor_energia_discreta < 1.0)
        {
            let mut sistema_final = sistema_trabajo.clone();
            sistema_final.valores = valores_finales;
            return Some(sistema_final.mapear_valores());
        }

        None
    }
}
