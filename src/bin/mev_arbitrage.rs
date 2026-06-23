use speculam_solver::{MotorSpeculam, Restriccion, SistemaRestricciones};
use std::time::Instant;

fn main() {
    let cyan = "\x1b[36;1m";
    let green = "\x1b[32;1m";
    let yellow = "\x1b[33;1m";
    let reset = "\x1b[0m";

    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!(
        "{}    S.P.E.C.U.L.A.M. v5 - OPTIMIZADOR MEV DE ARBITRAJE     {}",
        cyan, reset
    );
    println!(
        "{}=========================================================={}",
        cyan, reset
    );
    println!("  Calculando el tamaño de swap óptimo en microsegundos...\n");

    // =========================================================================
    // CASO DE USO: Arbitraje de Liquidez SOL/USDC entre ORCA y RAYDIUM
    // =========================================================================
    // Reservas de los Pools:
    // Pool A (Orca) - Precio SOL es más bajo (SOL subvaluado):
    //   - Reserva X (SOL) = 1,000.0
    //   - Reserva Y (USDC) = 150,000.0 (Precio implícito: 150.0 USDC por SOL)
    //   - K_Orca = 150,000,000.0
    //
    // Pool B (Raydium) - Precio SOL es más alto (SOL sobrevaluado):
    //   - Reserva Y (USDC) = 160,000.0
    //   - Reserva X (SOL) = 1,000.0 (Precio implícito: 160.0 USDC por SOL)
    //   - K_Raydium = 160,000,000.0

    let mut sistema = SistemaRestricciones::new();

    // 1. Definimos variables rígidas (Reserva inicial inalterable de los Pools)
    let x_orca = sistema.agregar_variable("X_Orca", 1000.0, 0.0);
    let y_orca = sistema.agregar_variable("Y_Orca", 150000.0, 0.0);
    let k_orca = sistema.agregar_variable("K_Orca", 150000000.0, 0.0);

    let x_raydium = sistema.agregar_variable("X_Raydium", 1000.0, 0.0);
    let y_raydium = sistema.agregar_variable("Y_Raydium", 160000.0, 0.0);
    let k_raydium = sistema.agregar_variable("K_Raydium", 160000000.0, 0.0);

    // 2. Definimos variables elásticas (Mapean el flujo físico del swap)
    //   - DeltaY: Entrada de USDC a Orca. Inicializamos en 1000.0 USDC.
    //   - DeltaX: Salida de SOL de Orca y entrada de SOL a Raydium.
    //   - DeltaY_Out: Salida final de USDC de Raydium.
    let delta_y = sistema.agregar_variable("DeltaY", 1000.0, 1.0);
    let delta_x = sistema.agregar_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = sistema.agregar_variable("DeltaY_Out", 1000.0, 1.0);

    // Estados intermedios elásticos de los pools post-trade
    let x_orca_post = sistema.agregar_variable("X_Orca_Post", 994.0, 1.0);
    let y_orca_post = sistema.agregar_variable("Y_Orca_Post", 151000.0, 1.0);

    let x_raydium_post = sistema.agregar_variable("X_Raydium_Post", 1006.0, 1.0);
    let y_raydium_post = sistema.agregar_variable("Y_Raydium_Post", 159000.0, 1.0);

    // 3. Escribimos las restricciones físicas de la curva AMM x * y = k
    //
    // Orca: (Y_Orca + DeltaY) = Y_Orca_Post
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "orca_suma_y".to_string(),
        sumandos: vec![y_orca, delta_y],
        resultado: y_orca_post,
    });
    // Orca: (X_Orca - DeltaX) = X_Orca_Post  =>  X_Orca_Post + DeltaX = X_Orca
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "orca_resta_x".to_string(),
        sumandos: vec![x_orca_post, delta_x],
        resultado: x_orca,
    });
    // Orca: X_Orca_Post * Y_Orca_Post = K_Orca
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "orca_amm".to_string(),
        factores: vec![x_orca_post, y_orca_post],
        resultado: k_orca,
    });

    // Raydium: (X_Raydium + DeltaX) = X_Raydium_Post
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "ray_suma_x".to_string(),
        sumandos: vec![x_raydium, delta_x],
        resultado: x_raydium_post,
    });
    // Raydium: (Y_Raydium - DeltaY_Out) = Y_Raydium_Post  =>  Y_Raydium_Post + DeltaY_Out = Y_Raydium
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "ray_resta_y".to_string(),
        sumandos: vec![y_raydium_post, delta_y_out],
        resultado: y_raydium,
    });
    // Raydium: X_Raydium_Post * Y_Raydium_Post = K_Raydium
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "ray_amm".to_string(),
        factores: vec![x_raydium_post, y_raydium_post],
        resultado: k_raydium,
    });

    // Rango de seguridad: no permitir swaps negativos
    sistema.agregar_restriccion(Restriccion::Rango {
        nombre: "limite_delta_y".to_string(),
        variable: delta_y,
        min: 0.0,
        max: 50000.0,
    });

    // 4. Potencial Atractor de Ganancia (Maximizar DeltaY_Out - DeltaY)
    //   Exigimos un objetivo ambicioso: DeltaY_Out = DeltaY + 50000.0 (ganar 50k USDC).
    //   La fuerza de atracción guiará a DeltaY hacia el tamaño de swap óptimo exacto
    //   donde el deslizamiento (slippage) del pool neutraliza el beneficio marginal.
    let objetivo_ganancia = sistema.agregar_variable("Objetivo_Ganancia", 50000.0, 0.0); // Fija
    let delta_y_mas_objetivo = sistema.agregar_variable("DeltaY_Mas_Objetivo", 51000.0, 1.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_objetivo".to_string(),
        sumandos: vec![delta_y, objetivo_ganancia],
        resultado: delta_y_mas_objetivo,
    });

    // Fuerza atractora de ganancia
    sistema.agregar_restriccion(Restriccion::IgualdadDirecta {
        nombre: "atraccion_ganancia".to_string(),
        var_a: delta_y_out,
        var_b: delta_y_mas_objetivo,
    });

    // Precalcular adyacencias
    sistema.precalcular_adyacencias();

    // 5. Ejecutar la optimización elástica
    println!(
        "{}---> Buscando tamaño óptimo de swap con S.P.E.C.U.L.A.M. v5...{}",
        yellow, reset
    );
    let t_solve_inicio = Instant::now();
    let motor = MotorSpeculam::new();
    let solucion = motor.evaluar(&sistema);
    let t_solve_elapsed = t_solve_inicio.elapsed();

    println!("{}>>> CÁLCULO DE ARBITRAJE COMPLETADO <<<{}", green, reset);
    println!("  Tiempo de ejecución: {:.2?}\n", t_solve_elapsed);

    // 6. Analizar la solución matemática óptima encontrada
    match solucion {
        speculam_solver::SolucionEspejo::Pista {
            valores_ajustados,
            explicacion,
            ..
        } => {
            let dx = valores_ajustados.get("DeltaX").unwrap();
            let dy = valores_ajustados.get("DeltaY").unwrap();
            let dy_out = valores_ajustados.get("DeltaY_Out").unwrap();
            let ganancia = dy_out - dy;

            println!("{}", explicacion);

            println!("  [ÓPTIMO DE TRADE ENCONTRADO]");
            println!("  - Inyectar en Orca (USDC): {:.2} USDC", dy);
            println!(
                "  - Retirar de Orca (SOL): {:.4} SOL (Precio efectivo: {:.2} USDC/SOL)",
                dx,
                dy / dx
            );
            println!("  - Retirar de Raydium (USDC): {:.2} USDC", dy_out);
            println!(
                "  - Ganancia neta estimada: {}${:.2} USDC{}",
                green, ganancia, reset
            );
            println!("  - SOL intermedio movido: {:.4} SOL", dx);
        }
        speculam_solver::SolucionEspejo::Directa { valores } => {
            println!("Solución directa sin tensión.");
            println!("Valores: {:?}", valores);
        }
        speculam_solver::SolucionEspejo::ComplejidadAlta { estres, mensaje } => {
            println!("Error al optimizar: {}", mensaje);
            println!("Estrés: {}", estres.energia_total);
        }
    }

    println!(
        "\n{}=========================================================={}",
        cyan, reset
    );
}
