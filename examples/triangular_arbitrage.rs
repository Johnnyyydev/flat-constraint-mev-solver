use speculam_solver::{MotorSpeculam, Restriccion, SistemaRestricciones, SolucionEspejo};

fn main() {
    println!("=== S.P.E.C.U.L.A.M. Ejemplo: Arbitraje Triangular ===");

    let mut sistema = SistemaRestricciones::new();

    // Pool 1: SOL/USDC (USDC -> SOL) - 1,000 SOL / 150,000 USDC
    let x1 = sistema.agregar_variable("X1", 1000.0, 0.0);
    let y1 = sistema.agregar_variable("Y1", 150000.0, 0.0);
    let k1 = sistema.agregar_variable("K1", 150000000.0, 0.0);

    // Pool 2: SOL/BONK (SOL -> BONK) - 1,000 SOL / 10,000,000 BONK
    let x2 = sistema.agregar_variable("X2", 1000.0, 0.0);
    let z2 = sistema.agregar_variable("Z2", 10000000.0, 0.0);
    let k2 = sistema.agregar_variable("K2", 10000000000.0, 0.0);

    // Pool 3: BONK/USDC (BONK -> USDC) - 10,000,000 BONK / 160,000 USDC
    let z3 = sistema.agregar_variable("Z3", 10000000.0, 0.0);
    let y3 = sistema.agregar_variable("Y3", 160000.0, 0.0);
    let k3 = sistema.agregar_variable("K3", 1600000000000.0, 0.0);

    // Variables de swap (flujos)
    let delta_y_in = sistema.agregar_variable("DeltaY_In", 1000.0, 1.0);
    let delta_x1 = sistema.agregar_variable("DeltaX1", 6.0, 1.0);
    let delta_z2 = sistema.agregar_variable("DeltaZ2", 60000.0, 1.0);
    let delta_y_out = sistema.agregar_variable("DeltaY_Out", 1000.0, 1.0);

    // Reservas post-swap
    let x1_post = sistema.agregar_variable("X1_Post", 994.0, 1.0);
    let y1_post = sistema.agregar_variable("Y1_Post", 151000.0, 1.0);

    let x2_post = sistema.agregar_variable("X2_Post", 1006.0, 1.0);
    let z2_post = sistema.agregar_variable("Z2_Post", 9940000.0, 1.0);

    let z3_post = sistema.agregar_variable("Z3_Post", 10060000.0, 1.0);
    let y3_post = sistema.agregar_variable("Y3_Post", 159000.0, 1.0);

    // Pool 1 (USDC -> SOL)
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p1_suma_y".to_string(),
        sumandos: vec![y1, delta_y_in],
        resultado: y1_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p1_resta_x".to_string(),
        sumandos: vec![x1_post, delta_x1],
        resultado: x1,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "p1_amm".to_string(),
        factores: vec![x1_post, y1_post],
        resultado: k1,
    });

    // Pool 2 (SOL -> BONK)
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p2_suma_x".to_string(),
        sumandos: vec![x2, delta_x1],
        resultado: x2_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p2_resta_z".to_string(),
        sumandos: vec![z2_post, delta_z2],
        resultado: z2,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "p2_amm".to_string(),
        factores: vec![x2_post, z2_post],
        resultado: k2,
    });

    // Pool 3 (BONK -> USDC)
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p3_suma_z".to_string(),
        sumandos: vec![z3, delta_z2],
        resultado: z3_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "p3_resta_y".to_string(),
        sumandos: vec![y3_post, delta_y_out],
        resultado: y3,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "p3_amm".to_string(),
        factores: vec![z3_post, y3_post],
        resultado: k3,
    });

    // Objetivo de Ganancia elástica (Maximizar DeltaY_Out - DeltaY_In)
    let objetivo_ganancia = sistema.agregar_variable("Objetivo_Ganancia", 50000.0, 0.0);
    let delta_y_mas_objetivo = sistema.agregar_variable("DeltaY_Mas_Objetivo", 51000.0, 1.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_objetivo".to_string(),
        sumandos: vec![delta_y_in, objetivo_ganancia],
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
            let dy_in = valores_ajustados.get("DeltaY_In").unwrap();
            let dy_out = valores_ajustados.get("DeltaY_Out").unwrap();
            println!("¡Arbitraje triangular resuelto con éxito!");
            println!("USDC Entrada: {:.2} USDC", dy_in);
            println!("USDC Salida: {:.2} USDC", dy_out);
            println!("Ganancia Neta: {:.2} USDC", dy_out - dy_in);
        }
        _ => println!("Error al optimizar el arbitraje triangular."),
    }
}
