use speculam_solver::{
    Autogenesis, MotorSpeculam, QuantumJumper, Restriccion, SistemaRestricciones, SolucionEspejo,
};

#[test]
fn test_sistema_coherente_directo() {
    let mut sistema = SistemaRestricciones::new();
    let x = sistema.agregar_variable("x", 5.0, 0.0);
    let y = sistema.agregar_variable("y", 10.0, 0.0);
    let z = sistema.agregar_variable("z", 15.0, 0.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "x_mas_y_eq_z".to_string(),
        sumandos: vec![x, y],
        resultado: z,
    });

    let motor = MotorSpeculam::new();
    sistema.precalcular_adyacencias();
    let solucion = motor.evaluar(&sistema);

    assert!(matches!(solucion, SolucionEspejo::Directa { .. }));
}

#[test]
fn test_contradiccion_rigida_genera_pista() {
    let mut sistema = SistemaRestricciones::new();
    let a = sistema.agregar_variable("A", 10.0, 0.0);
    let b = sistema.agregar_variable("B", 1.0, 0.0);
    let c = sistema.agregar_variable("C", 12.0, 0.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_invalida".to_string(),
        sumandos: vec![a, b],
        resultado: c,
    });

    let motor = MotorSpeculam::new();
    sistema.precalcular_adyacencias();
    let solucion = motor.evaluar(&sistema);

    if let SolucionEspejo::Pista {
        tensiones_residuales,
        ..
    } = solucion
    {
        let tension = tensiones_residuales.get("suma_invalida").unwrap();
        assert!((tension - (-1.0)).abs() < 1e-5);
    } else {
        panic!("Se esperaba una sugerencia de colapso de fase");
    }
}

#[test]
fn test_relajacion_elastica() {
    let mut sistema = SistemaRestricciones::new();
    let a = sistema.agregar_variable("A", 10.0, 0.0);
    let b = sistema.agregar_variable("B", 1.0, 0.0);
    let x = sistema.agregar_variable("X", 12.0, 1.0);

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "suma_relajable".to_string(),
        sumandos: vec![a, b],
        resultado: x,
    });

    let motor = MotorSpeculam::new();
    sistema.precalcular_adyacencias();
    let solucion = motor.evaluar(&sistema);

    if let SolucionEspejo::Pista {
        valores_ajustados,
        desviaciones,
        ..
    } = solucion
    {
        let x_final = valores_ajustados.get("X").unwrap();
        assert!((x_final - 11.0).abs() < 1e-4);

        let x_desviacion = desviaciones.get("X").unwrap();
        assert!((x_desviacion - (-1.0)).abs() < 1e-4);
    } else {
        panic!("Se esperaba una relajación elástica");
    }
}

#[test]
fn test_autogenesis_y_resolucion() {
    let flujo_crudo = r#"
        fixed(A, 5.0)
        fixed(B, 3.0)
        var(X, 10.0)
        A * B = X
    "#;

    let sistema = Autogenesis::compilar_flujo_crudo(flujo_crudo).unwrap();
    let motor = MotorSpeculam::new();
    let solucion = motor.evaluar(&sistema);

    if let SolucionEspejo::Pista {
        valores_ajustados, ..
    } = solucion
    {
        let x_final = valores_ajustados.get("X").unwrap();
        // X debe aproximarse a 5 * 3 = 15
        assert!((x_final - 15.0).abs() < 1e-4);
    } else {
        panic!("Se esperaba una relajación de X a 15");
    }
}

#[test]
fn test_quantum_jump_resolucion() {
    let mut sistema = SistemaRestricciones::new();
    let factor = sistema.agregar_variable("Factor", 7.0, 0.0);
    let target = sistema.agregar_variable("Target", 581.0, 0.0);
    let nonce = sistema.agregar_variable("Nonce", 10.0, 1.0);

    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "ecuacion_nonce".to_string(),
        factores: vec![nonce, factor],
        resultado: target,
    });

    let jumper = QuantumJumper::new();
    sistema.precalcular_adyacencias();
    let solucion = jumper.saltar_espacio_discreto(&sistema, &[nonce]).unwrap();

    let nonce_final = solucion.get("Nonce").unwrap();
    assert!((nonce_final - 83.0).abs() < 1e-5);
}

#[test]
fn test_arbitraje_mev_orca_raydium() {
    let mut sistema = SistemaRestricciones::new();

    // Pool A (Orca) - SOL subvaluado (150 USDC/SOL)
    let x_orca = sistema.agregar_variable("X_Orca", 1000.0, 0.0);
    let y_orca = sistema.agregar_variable("Y_Orca", 150000.0, 0.0);
    let k_orca = sistema.agregar_variable("K_Orca", 150000000.0, 0.0);

    // Pool B (Raydium) - SOL sobrevaluado (160 USDC/SOL)
    let x_raydium = sistema.agregar_variable("X_Raydium", 1000.0, 0.0);
    let y_raydium = sistema.agregar_variable("Y_Raydium", 160000.0, 0.0);
    let k_raydium = sistema.agregar_variable("K_Raydium", 160000000.0, 0.0);

    // Variables de swap
    let delta_y = sistema.agregar_variable("DeltaY", 1000.0, 1.0);
    let delta_x = sistema.agregar_variable("DeltaX", 6.0, 1.0);
    let delta_y_out = sistema.agregar_variable("DeltaY_Out", 1000.0, 1.0);

    let x_orca_post = sistema.agregar_variable("X_Orca_Post", 994.0, 1.0);
    let y_orca_post = sistema.agregar_variable("Y_Orca_Post", 151000.0, 1.0);

    let x_raydium_post = sistema.agregar_variable("X_Raydium_Post", 1006.0, 1.0);
    let y_raydium_post = sistema.agregar_variable("Y_Raydium_Post", 159000.0, 1.0);

    // Restricciones físicas
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "orca_suma_y".to_string(),
        sumandos: vec![y_orca, delta_y],
        resultado: y_orca_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "orca_resta_x".to_string(),
        sumandos: vec![x_orca_post, delta_x],
        resultado: x_orca,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "orca_amm".to_string(),
        factores: vec![x_orca_post, y_orca_post],
        resultado: k_orca,
    });

    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "ray_suma_x".to_string(),
        sumandos: vec![x_raydium, delta_x],
        resultado: x_raydium_post,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadSuma {
        nombre: "ray_resta_y".to_string(),
        sumandos: vec![y_raydium_post, delta_y_out],
        resultado: y_raydium,
    });
    sistema.agregar_restriccion(Restriccion::IgualdadProducto {
        nombre: "ray_amm".to_string(),
        factores: vec![x_raydium_post, y_raydium_post],
        resultado: k_raydium,
    });

    // Rango límite
    sistema.agregar_restriccion(Restriccion::Rango {
        nombre: "limite_delta_y".to_string(),
        variable: delta_y,
        min: 0.0,
        max: 50000.0,
    });

    // Atractor de ganancia
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

    if let SolucionEspejo::Pista {
        valores_ajustados, ..
    } = solucion
    {
        let dy = valores_ajustados.get("DeltaY").unwrap();
        let dy_out = valores_ajustados.get("DeltaY_Out").unwrap();
        let ganancia = dy_out - dy;

        // Comprobar que el arbitraje da ganancia positiva
        assert!(
            ganancia > 50.0,
            "La ganancia optimizada debería ser sustancial (> 50 USDC)"
        );
        assert!(
            ganancia < 5000.0,
            "La ganancia debe estar acotada por el deslizamiento"
        );

        // Comprobar que el tamaño del swap óptimo convergió cerca de 950.0 USDC
        assert!((dy - 950.92).abs() < 50.0);
    } else {
        panic!("Se esperaba solución elástica de tipo Pista");
    }
}

#[test]
fn test_arbitraje_triangular_multi_hop() {
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

    // Variables de swap
    let delta_y_in = sistema.agregar_variable("DeltaY_In", 1000.0, 1.0);
    let delta_x1 = sistema.agregar_variable("DeltaX1", 6.0, 1.0);
    let delta_z2 = sistema.agregar_variable("DeltaZ2", 60000.0, 1.0);
    let delta_y_out = sistema.agregar_variable("DeltaY_Out", 1000.0, 1.0);

    // Post-reserves
    let x1_post = sistema.agregar_variable("X1_Post", 994.0, 1.0);
    let y1_post = sistema.agregar_variable("Y1_Post", 151000.0, 1.0);

    let x2_post = sistema.agregar_variable("X2_Post", 1006.0, 1.0);
    let z2_post = sistema.agregar_variable("Z2_Post", 9940000.0, 1.0);

    let z3_post = sistema.agregar_variable("Z3_Post", 10060000.0, 1.0);
    let y3_post = sistema.agregar_variable("Y3_Post", 159000.0, 1.0);

    // Restricciones de Pool 1
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

    // Restricciones de Pool 2
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

    // Restricciones de Pool 3
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

    // Atractor de ganancia
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

    if let SolucionEspejo::Pista {
        valores_ajustados, ..
    } = solucion
    {
        let dy_in = valores_ajustados.get("DeltaY_In").unwrap();
        let dy_out = valores_ajustados.get("DeltaY_Out").unwrap();
        let ganancia = dy_out - dy_in;

        // Comprobar que el arbitraje triangular da ganancia positiva
        assert!(
            ganancia > 10.0,
            "La ganancia triangular debería ser positiva (> 10 USDC)"
        );
        assert!(ganancia < 5000.0);
    } else {
        panic!("Se esperaba solución elástica de tipo Pista para arbitraje triangular");
    }
}
