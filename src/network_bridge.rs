use crate::espejo::{MotorSpeculam, SolucionEspejo};
use crate::grafo::SistemaRestricciones;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Puente de red asíncrono que conecta el resolvedor elástico con flujos continuos de telemetría y red.
pub struct NetworkBridge;

impl NetworkBridge {
    /// Inicia una tarea asíncrona que simula un feed continuo de telemetría de red DePIN.
    /// Emite fluctuaciones de red cada cierto intervalo de milisegundos.
    pub fn iniciar_stream_simulado(intervalo_ms: u64) -> mpsc::Receiver<(String, f64)> {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let eventos = [
                ("Nodo_Origen".to_string(), 8.0),
                ("Fuga_Red".to_string(), 2.0),
                ("Nodo_Origen".to_string(), 4.0),
                ("Fuga_Red".to_string(), 0.5),
                ("Nodo_Origen".to_string(), 6.5),
                ("Fuga_Red".to_string(), 1.2),
            ];

            let mut idx = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(intervalo_ms)).await;

                let (var_name, val) = &eventos[idx % eventos.len()];
                if tx.send((var_name.clone(), *val)).await.is_err() {
                    break; // El receptor se cerró, finalizamos la simulación
                }

                idx += 1;
            }
        });

        rx
    }

    /// Escucha el flujo asíncrono, actualiza en caliente los valores en la memoria plana del sistema
    /// y ejecuta re-evaluaciones de estrés ultra-rápidas en milisegundos.
    pub async fn procesar_flujo(
        mut rx: mpsc::Receiver<(String, f64)>,
        mut sistema: SistemaRestricciones,
    ) {
        let motor = MotorSpeculam::new();
        let cyan = "\x1b[36m";
        let green = "\x1b[32m";
        let yellow = "\x1b[33m";
        let reset = "\x1b[0m";

        println!(
            "  [BRIDGE] Escuchando telemetría asíncrona... listo para disipar estrés en vivo.\n"
        );

        while let Some((nombre_var, nuevo_valor)) = rx.recv().await {
            println!(
                "{}----------------------------------------------------------{}",
                cyan, reset
            );
            println!(
                "  [TELEMETRÍA EN VIVO] Ingesta de red: {}{} = {:.2}{}",
                yellow, nombre_var, nuevo_valor, reset
            );

            // 1. Buscar la variable en caliente por O(1) e inyectar el nuevo valor en la memoria plana
            if let Some(&var_idx) = sistema.variable_indices.get(&nombre_var) {
                let valor_anterior = sistema.valores[var_idx];
                sistema.valores[var_idx] = nuevo_valor;

                println!(
                    "  [MEMORIA PLANA] Variable '{}' actualizada en caliente: {:.2} -> {:.2}",
                    nombre_var, valor_anterior, nuevo_valor
                );

                // 2. Ejecutar re-evaluación elástica paralela de manera inmediata
                let t_solve_inicio = Instant::now();
                let solucion = motor.evaluar(&sistema);
                let t_solve_elapsed = t_solve_inicio.elapsed();

                println!(
                    "  [SPECULAM SOLVER] Re-equilibrio completado en: {}{:.2?}{}",
                    green, t_solve_elapsed, reset
                );

                // 3. Mostrar la disipación del estrés y valores resultantes
                match solucion {
                    SolucionEspejo::Pista {
                        valores_ajustados,
                        desviaciones,
                        ..
                    } => {
                        if !desviaciones.is_empty() {
                            println!("  [DEFORMACIONES]");
                            for (var, delta) in &desviaciones {
                                println!(
                                    "    - '{}' se ajustó por [{:+.4}] -> valor final: {:.4}",
                                    var,
                                    delta,
                                    valores_ajustados.get(var).unwrap()
                                );
                            }
                        } else {
                            println!(
                                "  [ESTADO] El cambio fue absorbido completamente por el sistema sin deformación residual."
                            );
                        }
                    }
                    SolucionEspejo::Directa { valores } => {
                        println!("  [ESTADO] Equilibrio perfecto directo. Valores:");
                        for (var, val) in &valores {
                            println!("    - '{}' = {:.4}", var, val);
                        }
                    }
                    SolucionEspejo::ComplejidadAlta { estres, mensaje } => {
                        println!("  [ADVERTENCIA] Caos detectado: {}", mensaje);
                        println!("    Tensión final del sistema: {:.4}", estres.energia_total);
                    }
                }
            } else {
                println!(
                    "  [ADVERTENCIA] Telemetría omitida: '{}' no pertenece al grafo lógico actual.",
                    nombre_var
                );
            }
            println!(
                "{}----------------------------------------------------------{}",
                cyan, reset
            );
        }
    }
}
