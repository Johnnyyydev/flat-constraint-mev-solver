use speculam_solver::{MotorSpeculam, Restriccion, SistemaRestricciones, SolucionEspejo};

fn main() {
    println!("=== S.P.E.C.U.L.A.M. Ejemplo: Arbitraje Simple ===");

    let mut sistema = SistemaRestricciones::new();

    // Pool A (Comprar SOL barato)
    // Reservas iniciales: X (SOL) = 1,000.0, Y (USDC) = 150,000.0
    let x_a = sistema.agregar_variable("X_A", 1000.0, 0.0);
    let y_a = sistema.agregar_variable("Y_A", 150000.0, 0.0);
    let k_a = sistema.agregar_variable("K_A", 150000000.0, 0.0);

    // Pool B (Vender SOL caro)
    // Reservas iniciales: X (SOL) = 1,000.0, Y (USDC) = 160,000.0
    let x_b = sistema.agregar_variable("X_B", 1000.0, 0.0);
    let y_b = sistema.agregar_variable("Y_B", 160000.0, 0.0);
    let k_b = sistema.agregar_variable("K_B", 160000000.0, 0.0);

    // Variables de swap (flujo de tokens)
    let delta_y = sistema.agregar_variable("DeltaY", 1000.0, 1.0);
    let delta_x = sistema.agregar_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = sistema.agregar_variable("DeltaY_Out", 1000.0, 1.0);

    // Estados intermedios tras el swap
    let x_a_post = sistema.agregar_variable("X_A_Post", 994.0, 1.0);
    let y_a_post = sistema.agregar_variable("Y_A_Post", 151000.0, 1.0);

    let x_b_post = sistema.agregar_variable("X_B_Post", 1006.0, 1.0);
    let y_b_post = sistema.agregar_variable("Y_B_Post", 159000.0, 1.0);

    // Restricciones físicas de Pool A (USDC -> SOL)
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "pool_a_y".to_string(),
        sumandos: vec![y_a, delta_y],
        resultado: y_a_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "pool_a_x".to_string(),
        sumandos: vec![x_a_post, delta_x],
        resultado: x_a,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "pool_a_amm".to_string(),
        factores: vec![x_a_post, y_a_post],
        resultado: k_a,
    });

    // Restricciones físicas de Pool B (SOL -> USDC)
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "pool_b_x".to_string(),
        sumandos: vec![x_b, delta_x],
        resultado: x_b_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "pool_b_y".to_string(),
        sumandos: vec![y_b_post, delta_y_out],
        resultado: y_b,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "pool_b_amm".to_string(),
        factores: vec![x_b_post, y_b_post],
        resultado: k_b,
    });

    // Objetivo: maximizar ganancia elástica
    let objetivo_ganancia = sistema.agregar_variable("Objetivo_Ganancia", 50000.0, 0.0);
    let delta_y_mas_objetivo = sistema.agregar_variable("DeltaY_Mas_Objetivo", 51000.0, 1.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_objetivo".to_string(),
        sumandos: vec![delta_y, objetivo_ganancia],
        resultado: delta_y_mas_objetivo,
    });

    sistema.agregar_restriccion(Restriccion::IgualdadDirecta {
        nombre: "atraccion_ganancia".to_string(),
        var_a: delta_y_out,
        var_b: delta_y_mas_objetivo,
    });

    sistema.precalcular_adyacencias();

    let motor = MotorSpeculam::new();
    let solucion = motor.evaluar(&sistema);

    match solucion {
        SolucionEspejo::Pista {
            valores_ajustados, ..
        } => {
            let dy = valores_ajustados.get("DeltaY").unwrap();
            let dy_out = valores_ajustados.get("DeltaY_Out").unwrap();
            println!("¡Arbitraje resuelto con éxito!");
            println!("USDC Inyectado: {:.2} USDC", dy);
            println!("USDC Obtenido: {:.2} USDC", dy_out);
            println!("Ganancia Neta: {:.2} USDC", dy_out - dy);
        }
        _ => println!("Error al optimizar el arbitraje."),
    }
}
