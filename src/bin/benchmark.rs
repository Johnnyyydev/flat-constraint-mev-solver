use std::time::Instant;
use speculam_solver::{Autogenesis, MotorSpeculam, CampoEstres};

fn main() {
    let cyan = "\x1b[36;1m";
    let green = "\x1b[32;1m";
    let yellow = "\x1b[33;1m";
    let reset = "\x1b[0m";

    println!("{}=========================================================={}", cyan, reset);
    println!("{}       S.P.E.C.U.L.A.M. BENCHMARK INDUSTRIAL (Paso 1)      {}", cyan, reset);
    println!("{}=========================================================={}", cyan, reset);
    println!("  Generando red sintética masiva de variables y restricciones...\n");

    let num_variables = 10000;
    let num_bloques = 3300; // Cada bloque define 3 variables y 3 restricciones cruzadas

    // 1. Generar flujo crudo de texto para Autogénesis en memoria
    let t_gen_inicio = Instant::now();
    let mut raw_data = String::new();

    // Definición de variables
    for i in 0..num_variables {
        if i % 10 == 0 {
            // Cada 10 variables, fijamos una para actuar como condición de contorno
            raw_data.push_str(&format!("fixed(V_{}, {}.0)\n", i, (i % 5) + 1));
        } else if i % 10 == 3 {
            // Algunas variables con elasticidad específica
            raw_data.push_str(&format!("elastic(V_{}, 1.0, 1.5)\n", i));
        } else {
            // El resto son variables elásticas estándar
            raw_data.push_str(&format!("var(V_{}, 0.0)\n", i));
        }
    }

    // Definición de restricciones interconectadas en cascada (para formar un campo de estrés global)
    for b in 0..num_bloques {
        let v0 = b * 3;
        let v1 = b * 3 + 1;
        let v2 = b * 3 + 2;
        let v_next = (b * 3 + 3) % num_variables;

        if v2 < num_variables {
            // Ecuación de Suma: V0 + V1 = V2
            raw_data.push_str(&format!("V_{} + V_{} = V_{}\n", v0, v1, v2));
            // Ecuación de Producto acoplada: V1 * V2 = V_next
            raw_data.push_str(&format!("V_{} * V_{} = V_{}\n", v1, v2, v_next));
            // Rango limitador en la variable intermedia
            raw_data.push_str(&format!("rango(V_{}, 0.0, 50.0)\n", v1));
        }
    }

    let t_gen_elapsed = t_gen_inicio.elapsed();
    println!("• Generación de datos sintéticos: {} variable(s) y {} ecuación(es) en texto plano.", num_variables, num_bloques * 3);
    println!("  Tiempo: {:.2?}\n", t_gen_elapsed);

    // 2. Compilar con Autogénesis
    println!("{}---> Iniciando Compilación por Autogénesis...{}", yellow, reset);
    let t_comp_inicio = Instant::now();
    let sistema = match Autogenesis::compilar_flujo_crudo(&raw_data) {
        Ok(s) => s,
        Err(e) => {
            println!("Error al compilar el grafo: {}", e);
            return;
        }
    };
    let t_comp_elapsed = t_comp_inicio.elapsed();
    println!("{}>>> COMPILACIÓN COMPLETADA <<<{}", green, reset);
    println!("  Variables cargadas en memoria plana: {}", sistema.valores.len());
    println!("  Restricciones compiladas: {}", sistema.restricciones.len());
    println!("  Tiempo de compilación: {:.2?}\n", t_comp_elapsed);

    // Calcular estrés inicial
    let estres_inicial = CampoEstres::calcular(&sistema);
    println!("• Energía elástica inicial del sistema: {:.4}", estres_inicial.energia_total);

    // 3. Resolver con el motor Speculam v3 optimizado para memoria plana
    println!("\n{}---> Ejecutando resolvedor matricial plano (Speculam Solver)...{}", yellow, reset);
    let motor = MotorSpeculam::new();
    let t_solve_inicio = Instant::now();
    let solucion = motor.evaluar(&sistema);
    let t_solve_elapsed = t_solve_inicio.elapsed();

    println!("{}>>> RESOLUCIÓN COMPLETADA <<<{}", green, reset);
    println!("  Tiempo de ejecución del resolvedor: {:.2?}", t_solve_elapsed);

    // Evaluar energía residual
    match solucion {
        speculam_solver::SolucionEspejo::Pista { tensiones_residuales, valores_ajustados, .. } => {
            let mut verificador = sistema.clone();
            for (var, val) in valores_ajustados {
                if let Some(&idx) = verificador.variable_indices.get(&var) {
                    verificador.valores[idx] = val;
                }
            }
            let estres_final = CampoEstres::calcular(&verificador);
            println!("• Energía elástica final: {:.6}", estres_final.energia_total);
            println!("  Tensiones de ecuaciones activas (no resueltas por rigidez): {}", tensiones_residuales.len());
            println!("  Reducción de estrés: {:.2}%", 
                (1.0 - estres_final.energia_total / estres_inicial.energia_total) * 100.0
            );
        },
        speculam_solver::SolucionEspejo::Directa { .. } => {
            println!("• Energía elástica final: 0.000000 (Solución directa perfecta).");
        },
        speculam_solver::SolucionEspejo::ComplejidadAlta { estres, mensaje } => {
            println!("• Estado de salida: Complejidad Fuera de Límites");
            println!("  Mensaje: {}", mensaje);
            println!("  Energía elástica final: {:.6}", estres.energia_total);
        }
    }

    println!("\n{}=========================================================={}", cyan, reset);
}
