use std::collections::HashMap;

/// Las restricciones en representación plana usan índices `usize`
/// en lugar de cadenas de texto para evitar búsquedas lentas en mapas hash.
#[derive(Debug, Clone)]
pub enum Restriccion {
    /// sumandos[0] + sumandos[1] + ... = resultado
    IgualdadSuma {
        nombre: String,
        sumandos: Vec<usize>,
        resultado: usize,
    },
    /// factores[0] * factores[1] * ... = resultado
    IgualdadProducto {
        nombre: String,
        factores: Vec<usize>,
        resultado: usize,
    },
    /// variable en rango [min, max]
    Rango {
        nombre: String,
        variable: usize,
        min: f64,
        max: f64,
    },
    /// var_a = var_b
    IgualdadDirecta {
        nombre: String,
        var_a: usize,
        var_b: usize,
    },
}

impl Restriccion {
    pub fn nombre(&self) -> &str {
        match self {
            Restriccion::IgualdadSuma { nombre, .. } => nombre,
            Restriccion::IgualdadProducto { nombre, .. } => nombre,
            Restriccion::Rango { nombre, .. } => nombre,
            Restriccion::IgualdadDirecta { nombre, .. } => nombre,
        }
    }

    /// Calcula la derivada parcial de la tensión de la restricción con respecto a la variable `var_idx`.
    /// Este método encapsula el comportamiento local de derivadas para la paralelización lock-free.
    pub fn derivada_parcial(&self, var_idx: usize, valores: &[f64]) -> f64 {
        match self {
            Restriccion::IgualdadSuma { sumandos, resultado, .. } => {
                let mut d = 0.0;
                for &s in sumandos {
                    if s == var_idx {
                        d += 1.0;
                    }
                }
                if *resultado == var_idx {
                    d -= 1.0;
                }
                d
            },
            Restriccion::IgualdadProducto { factores, resultado, .. } => {
                let mut d = 0.0;
                for (i, &f) in factores.iter().enumerate() {
                    if f == var_idx {
                        let mut prod = 1.0;
                        for (k, &other_f) in factores.iter().enumerate() {
                            if i != k {
                                prod *= valores[other_f];
                            }
                        }
                        d += prod;
                    }
                }
                if *resultado == var_idx {
                    d -= 1.0;
                }
                d
            },
            Restriccion::Rango { variable, min, max, .. } => {
                if *variable == var_idx {
                    let val = valores[*variable];
                    if val < *min || val > *max {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            },
            Restriccion::IgualdadDirecta { var_a, var_b, .. } => {
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

/// Sistema de restricciones matricial y plano.
/// Toda la información está almacenada en arrays contiguos en memoria,
/// listos para ser recorridos secuencialmente y permitir auto-vectorización (SIMD).
#[derive(Debug, Clone, Default)]
pub struct SistemaRestricciones {
    pub nombres: Vec<String>,
    pub valores: Vec<f64>,
    pub elasticidades: Vec<f64>,
    /// Mapeo auxiliar de nombre -> índice. Se utiliza SOLO durante la construcción
    /// o inserción del grafo, nunca durante el bucle de optimización interno.
    pub variable_indices: HashMap<String, usize>,
    pub restricciones: Vec<Restriccion>,
    /// Mapeo de adyacencia: para cada índice de variable, la lista de índices de restricciones que la referencian.
    /// Esto es crítico para la paralización gather multihilo.
    pub variables_a_restricciones: Vec<Vec<usize>>,
}

impl SistemaRestricciones {
    pub fn new() -> Self {
        SistemaRestricciones {
            nombres: Vec::new(),
            valores: Vec::new(),
            elasticidades: Vec::new(),
            variable_indices: HashMap::new(),
            restricciones: Vec::new(),
            variables_a_restricciones: Vec::new(),
        }
    }

    /// Agrega una variable al sistema plano y devuelve su índice único.
    /// Si la variable ya existe, actualiza sus propiedades.
    pub fn agregar_variable(&mut self, nombre: &str, valor: f64, elasticidad: f64) -> usize {
        let elasticidad_val = elasticidad.max(0.0);
        if let Some(&idx) = self.variable_indices.get(nombre) {
            self.valores[idx] = valor;
            self.elasticidades[idx] = elasticidad_val;
            idx
        } else {
            let idx = self.valores.len();
            self.nombres.push(nombre.to_string());
            self.valores.push(valor);
            self.elasticidades.push(elasticidad_val);
            self.variable_indices.insert(nombre.to_string(), idx);
            idx
        }
    }

    /// Obtiene el índice de una variable. Si no existe, la crea con valores por defecto.
    pub fn obtener_o_crear_variable(&mut self, nombre: &str) -> usize {
        if let Some(&idx) = self.variable_indices.get(nombre) {
            idx
        } else {
            self.agregar_variable(nombre, 0.0, 1.0)
        }
    }

    /// Agrega una restricción al grafo plano.
    pub fn agregar_restriccion(&mut self, restriccion: Restriccion) {
        self.restricciones.push(restriccion);
    }

    /// Precalcula el mapeo de adyacencia de variables a restricciones.
    /// Debe ser llamado después de agregar todas las variables y restricciones
    /// y antes de arrancar el bucle del resolvedor.
    pub fn precalcular_adyacencias(&mut self) {
        let num_vars = self.valores.len();
        let mut mapping = vec![Vec::new(); num_vars];

        for (r_idx, rest) in self.restricciones.iter().enumerate() {
            match rest {
                Restriccion::IgualdadSuma { sumandos, resultado, .. } => {
                    for &s in sumandos {
                        mapping[s].push(r_idx);
                    }
                    mapping[*resultado].push(r_idx);
                },
                Restriccion::IgualdadProducto { factores, resultado, .. } => {
                    for &f in factores {
                        mapping[f].push(r_idx);
                    }
                    mapping[*resultado].push(r_idx);
                },
                Restriccion::Rango { variable, .. } => {
                    mapping[*variable].push(r_idx);
                },
                Restriccion::IgualdadDirecta { var_a, var_b, .. } => {
                    mapping[*var_a].push(r_idx);
                    mapping[*var_b].push(r_idx);
                }
            }
        }

        // Eliminar duplicados para evitar cálculos redundantes si una variable
        // aparece múltiples veces en la misma restricción
        for list in &mut mapping {
            list.sort_unstable();
            list.dedup();
        }

        self.variables_a_restricciones = mapping;
    }

    /// Obtiene el valor actual de una variable por su índice de manera directa.
    #[inline(always)]
    pub fn obtener_valor(&self, idx: usize) -> f64 {
        self.valores[idx]
    }

    /// Actualiza el valor de una variable de manera directa.
    #[inline(always)]
    pub fn actualizar_valor(&mut self, idx: usize, nuevo_valor: f64) {
        self.valores[idx] = nuevo_valor;
    }

    /// Comprueba si la variable en un índice es rígida.
    #[inline(always)]
    pub fn es_fija(&self, idx: usize) -> bool {
        self.elasticidades[idx] <= f64::EPSILON
    }

    /// Devuelve el mapa de nombres a valores actuales (útil para recolectar resultados).
    pub fn mapear_valores(&self) -> HashMap<String, f64> {
        let mut mapa = HashMap::new();
        for (idx, nombre) in self.nombres.iter().enumerate() {
            mapa.insert(nombre.clone(), self.valores[idx]);
        }
        mapa
    }
}
