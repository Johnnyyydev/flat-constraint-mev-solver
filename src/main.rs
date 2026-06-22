use speculam_solver::{SistemaRestricciones, Restriccion, MotorSpeculam, SolucionEspejo, Autogenesis, QuantumJumper, CampoEstres, NetworkBridge};

#[tokio::main]
async fn main() {
    // Colores ANSI para la presentación
    let cyan = "\x1b[36;1m";
    let green = "\x1b[32;1m";
    let yellow = "\x1b[33;1m";
    let red = "\x1b[31;1m";
    let reset = "\x1b[0m";

    println!("{}=========================================================={}", cyan, reset);
    println!("{}    S.P.E.C.U.L.A.M. v5 - PUENTE DE RED ASÍNCRONO EN VIVO    {}", cyan, reset);
    println!("{}=========================================================={}", cyan, reset);
    println!("  Sistema Proactivo de Espejo para Colapsos Universales");
    println!("  y Lógica Avanzada Matricial en Rust (Tokio Runtime)\n");

    let motor = MotorSpeculam::new();

    // =========================================================================
    // ESCENARIO 1: Contradicción Pura e Inflexible (10 + 1 = 12)
    // =========================================================================
    println!("{}--- ESCENARIO 1: Contradicción Pura (10 + 1 = 12) ---{}", yellow, reset);
    println!("Variables fijas: A = 10, B = 1, C = 12");
    println!("Restricción: A + B = C (Memoria Contigua)\n");

    let mut sistema1 = SistemaRestricciones::new();
    let a1 = sistema1.agregar_variable("A", 10.0, 0.0);
    let b1 = sistema1.agregar_variable("B", 1.0, 0.0);
    let c1 = sistema1.agregar_variable("C", 12.0, 0.0);
    
    sistema1.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_base".to_string(),
        sumandos: vec![a1, b1],
        resultado: c1,
    });

    sistema1.precalcular_adyacencias();
    let analisis1 = motor.evaluar(&sistema1);
    mostrar_resultado(&analisis1, &cyan, &green, &red, &reset);

    // =========================================================================
    // ESCENARIO 2: Relajación Elástica (10 + 1 = X, con X maleable)
    // =========================================================================
    println!("\n{}--- ESCENARIO 2: Relajación Elástica (10 + 1 = X, X inicial = 12) ---{}", yellow, reset);
    println!("Variables: A = 10 (fija), B = 1 (fija), X = 12 (MALEABLE, elasticidad = 1.0)\n");

    let mut sistema2 = SistemaRestricciones::new();
    let a2 = sistema2.agregar_variable("A", 10.0, 0.0);
    let b2 = sistema2.agregar_variable("B", 1.0, 0.0);
    let x2 = sistema2.agregar_variable("X", 12.0, 1.0);
    
    sistema2.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_maleable".to_string(),
        sumandos: vec![a2, b2],
        resultado: x2,
    });

    sistema2.precalcular_adyacencias();
    let analisis2 = motor.evaluar(&sistema2);
    mostrar_resultado(&analisis2, &cyan, &green, &red, &reset);

    // =========================================================================
    // ESCENARIO 3: Grafo de Conflicto Cíclico
    // =========================================================================
    println!("\n{}--- ESCENARIO 3: Grafo de Conflicto Cíclico ---{}", yellow, reset);
    println!("Variables: A = 3.0 (fija), B = 4.0 (fija), Z = 5.0 (maleable, elasticidad = 1.0)");
    println!("Restricciones: A + B = Z  &  Z + B = 10\n");

    let mut sistema3 = SistemaRestricciones::new();
    let a3 = sistema3.agregar_variable("A", 3.0, 0.0);
    let b3 = sistema3.agregar_variable("B", 4.0, 0.0);
    let const_10 = sistema3.agregar_variable("Const_10", 10.0, 0.0);
    let z3 = sistema3.agregar_variable("Z", 5.0, 1.0);

    sistema3.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "relacion_A_B_Z".to_string(),
        sumandos: vec![a3, b3],
        resultado: z3,
    });
    
    sistema3.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "relacion_Z_B_10".to_string(),
        sumandos: vec![z3, b3],
        resultado: const_10,
    });

    sistema3.precalcular_adyacencias();
    let analisis3 = motor.evaluar(&sistema3);
    mostrar_resultado(&analisis3, &cyan, &green, &red, &reset);

    // =========================================================================
    // ESCENARIO 4: Autogénesis y Balance Elástico DePIN
    // =========================================================================
    println!("\n{}--- ESCENARIO 4: Autogénesis Autónoma e Ingesta Caótica DePIN ---{}", yellow, reset);
    let flujo_crudo = r#"
        // Definición de topología de red elástica
        fixed(Nodo_Origen, 5.0)
        elastic(Nodo_Puente, 2.0, 2.0)
        var(Nodo_Destino, 15.0)
        fixed(Fuga_Red, 1.0)
        var(Nodo_Salida, 20.0)

        // Relaciones físicas del flujo
        Nodo_Origen * Nodo_Puente = Nodo_Destino
        Nodo_Destino + Fuga_Red = Nodo_Salida
        rango(Nodo_Salida, 0.0, 10.0)
    "#;
    println!("Flujo de Entrada compilado autónomamente por el Autómata:\n{}", flujo_crudo);

    match Autogenesis::compilar_flujo_crudo(flujo_crudo) {
        Ok(sistema4) => {
            let analisis4 = motor.evaluar(&sistema4);
            mostrar_resultado(&analisis4, &cyan, &green, &red, &reset);
        },
        Err(e) => {
            println!("{}Error de Autogénesis: {}{}", red, e, reset);
        }
    }

    // =========================================================================
    // ESCENARIO 5: Búsqueda Discreta de Nonce (Saltador Cuántico)
    // =========================================================================
    println!("\n{}--- ESCENARIO 5: Búsqueda Discreta de Nonce (Saltador Cuántico) ---{}", yellow, reset);
    println!("Objetivo: Encontrar un nonce entero 'Nonce' tal que:");
    println!("  - Nonce * 7.0 = 581.0  => (Solución matemática discreta: 83)");
    println!("  - 'Nonce' se inicia en 10.0 y se le obliga elásticamente a cristalizar en entero.\n");

    let mut sistema5 = SistemaRestricciones::new();
    let factor = sistema5.agregar_variable("Factor", 7.0, 0.0);
    let target = sistema5.agregar_variable("Target", 581.0, 0.0);
    let nonce = sistema5.agregar_variable("Nonce", 10.0, 1.0); // Maleable

    sistema5.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "ecuacion_nonce".to_string(),
        factores: vec![nonce, factor],
        resultado: target,
    });

    let jumper = QuantumJumper::new();
    println!("Iniciando recocido periódico elástico de coordenadas discretas...");
    
    sistema5.precalcular_adyacencias();
    match jumper.saltar_espacio_discreto(&sistema5, &[nonce]) {
        Some(valores) => {
            println!("{}>>> ÉXITO: SALTO CUÁNTICO COMPLETADO <<<{}", green, reset);
            let nonce_val = valores.get("Nonce").unwrap();
            println!("  - 'Nonce' entero encontrado: {:.1}", nonce_val);
            
            // Verificación del estrés
            let mut verificador = sistema5.clone();
            verificador.valores[nonce] = *nonce_val;
            let estres_final = CampoEstres::calcular(&verificador);
            println!("  - Energía residual del sistema con nonce entero: {:.4}", estres_final.energia_total);
        },
        None => {
            println!("{}>>> ERROR: No se pudo cristalizar el nonce discreto <<<{}", red, reset);
        }
    }

    // =========================================================================
    // ESCENARIO 6: Puente de Red Asíncrono en Caliente (Ingesta en Vivo)
    // =========================================================================
    println!("\n{}--- ESCENARIO 6: Puente de Red Asíncrono (Ingesta e Integración en Vivo) ---{}", yellow, reset);
    println!("Compilando la topología de red base para enrutamiento DePIN...");
    
    let flujo_depin = r#"
        fixed(Nodo_Origen, 5.0)
        elastic(Nodo_Puente, 2.0, 2.0)
        var(Nodo_Destino, 15.0)
        fixed(Fuga_Red, 1.0)
        var(Nodo_Salida, 20.0)

        Nodo_Origen * Nodo_Puente = Nodo_Destino
        Nodo_Destino + Fuga_Red = Nodo_Salida
        rango(Nodo_Salida, 0.0, 10.0)
    "#;

    if let Ok(sistema_base) = Autogenesis::compilar_flujo_crudo(flujo_depin) {
        let rx = NetworkBridge::iniciar_stream_simulado(600); // Updates every 600ms
        
        println!("Iniciando procesamiento continuo en segundo plano (duración: 4 segundos)...");
        let bridge_handle = tokio::spawn(async move {
            NetworkBridge::procesar_flujo(rx, sistema_base).await;
        });

        // Dejar correr la simulación de telemetría por 4 segundos
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        bridge_handle.abort();
        println!("\n[MAIN] Stream de telemetría finalizado correctamente.");
    }

    println!("\n{}=========================================================={}", cyan, reset);
    println!("{}             FIN DEL ANÁLISIS DE AUTÓMATA ESPEJO v5       {}", cyan, reset);
    println!("{}=========================================================={}", cyan, reset);
}

fn mostrar_resultado(solucion: &SolucionEspejo, cyan: &str, green: &str, red: &str, reset: &str) {
    match solucion {
        SolucionEspejo::Directa { valores } => {
            println!("{}>>> SOLUCIÓN DIRECTA ENCONTRADA (Sin Tensión) <<<{}", green, reset);
            let mut ordenados: Vec<_> = valores.iter().collect();
            ordenados.sort_by_key(|a| a.0);
            for (var, val) in ordenados {
                println!("  - Variable '{}': {:.4}", var, val);
            }
        },
        SolucionEspejo::Pista { explicacion, valores_ajustados, .. } => {
            println!("{}>>> MENTE PROPIA: RUTA LOGICA ALTERNATIVA (COLAPSO DE FASE) <<<{}", cyan, reset);
            println!("{}", explicacion);
            println!("Valores finales en el equilibrio:");
            let mut ordenados: Vec<_> = valores_ajustados.iter().collect();
            ordenados.sort_by_key(|a| a.0);
            for (var, val) in ordenados {
                println!("  - '{}' = {:.4}", var, val);
            }
        },
        SolucionEspejo::ComplejidadAlta { estres, mensaje } => {
            println!("{}>>> ERROR: COMPLEJIDAD FUERA DE LÍMITES <<<{}", red, reset);
            println!("Mensaje: {}", mensaje);
            println!("Energía residual del sistema: {:.4}", estres.energia_total);
        }
    }
    println!();
}
