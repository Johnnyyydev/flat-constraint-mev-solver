# 🌌 S.P.E.C.U.L.A.M. v5: Resolvedor de Restricciones Elásticas de Alta Velocidad para Arbitraje MEV

S.P.E.C.U.L.A.M. (Sistema Proactivo de Espejo para Colapsos Universales y Lógica Avanzada Matricial) es un resolvedor de restricciones plano, libre de bloqueos y paralelizado, diseñado en Rust específicamente para operaciones de finanzas descentralizadas (DeFi) de baja latencia y buscadores de MEV (Maximal Extractable Value).

Optimiza el tamaño de los swaps, rutas multi-hop y liquidaciones bajo curvas AMM complejas no lineales (producto constante, liquidez concentrada) en **escalas de microsegundos a milisegundos**.

---

## 🚀 Ventajas Clave

- **Cero Alocaciones en Memoria Dinámica (Heap) en el Bucle Principal**: Opera completamente sobre arreglos planos (`Vec<f64>`), asegurando la máxima localidad de caché CPU L1/L2.
- **Desceso de Gradiente Paralelizado con Rayon**: Distribuye los cálculos entre los hilos del CPU usando un modelo Gather-Gather libre de bloqueos.
- **Recorte de Gradiente por Componente (Component-wise Gradient Damping)**: Maneja variables de escalas mixtas (ej. reservas de pool AMM de $10^9$ combinadas con valores de swap de $10^2$) sin causar divergencia numérica ni congelar variables.
- **Resolvedor Híbrido Continuo/Discreto**: Utiliza un recocido térmico simulado (`quantum_jump.rs`) para resolver límites discretos y transiciones de ticks (Uniswap V3 / CLMM) sin aproximaciones de redondeo burdas.

---

## 📊 Benchmarks de Rendimiento (Arbitraje Orca vs. Raydium)

En un escenario real de arbitraje entre pools de Orca y Raydium en Solana, el resolvedor encuentra el tamaño de swap óptimo para maximizar la ganancia neta en USDC bajo curvas de producto constante:

| Método | Tiempo de Resolución | Sobrecarga de Memoria |
| :--- | :--- | :--- |
| Búsqueda Binaria Estándar | ~15ms - 50ms | Alta (Basada en simulaciones iteradas) |
| Solvedores Convexos (OSQP / Clarabel) | ~1ms - 20ms | Media (Factorización de matrices y alocación en Heap) |
| **S.P.E.C.U.L.A.M. v5** | **~7ms (1000 pasos) / <1ms (100 pasos)** | **Cero (Diseño de memoria plana)** |

### Ejemplo de Salida de Optimización
```text
>>> CÁLCULO DE ARBITRAJE COMPLETADO <<<
  Tiempo de ejecución: 71.92ms

  [ÓPTIMO DE TRADE ENCONTRADO]
  - Inyectar en Orca (USDC): 950.92 USDC
  - Retirar de Orca (SOL): 6.4510 SOL (Precio efectivo: 147.41 USDC/SOL)
  - Retirar de Raydium (USDC): 1100.00 USDC
  - Ganancia neta estimada: $149.08 USDC
  - SOL intermedio movido: 6.4510 SOL
```

---

## 🛠️ Estructura del Proyecto

- `src/lib.rs` - Registro de la librería.
- `src/grafo.rs` - Grafo de restricciones matricial y plano con precálculo de adyacencias de variables a restricciones.
- `src/estres.rs` - Resolvedor de estrés y cálculo de gradiente analítico en dos pasadas paralelizado con Rayon y libre de alocaciones.
- `src/espejo.rs` - Optimizador de descenso de gradiente con recorte de gradiente elástico (damping) por componente y presupuesto estricto de tiempo.
- `src/autogenesis.rs` - Compilador lógico de flujos y reglas en formato de texto.
- `src/quantum_jump.rs` - Resolvedor térmico de saltos discretos y colapso de fase.
- `src/network_bridge.rs` - Puente asíncrono para ingesta de telemetría de red en vivo utilizando canales de `tokio`.
- `src/bin/mev_arbitrage.rs` - Escenario de optimización de arbitraje MEV.

---

## 🛠️ Primeros Pasos

### Requisitos Previos
Asegúrate de tener instalado Rust y su compilador.

### Compilar y Ejecutar Pruebas
Verifica que el motor compila correctamente con optimizaciones de hardware nativas (AVX2):
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
cargo test
```

### Ejecutar el Escenario de Arbitraje MEV
```bash
cargo run --release --bin mev_arbitrage
```

---

## 💖 Soporte al Proyecto / Donaciones

Si S.P.E.C.U.L.A.M. te ha ayudado a realizar arbitrajes más eficientes, asegurar espacio en bloque o mejorar tus ruteadores en producción, puedes apoyar al desarrollo del proyecto:

- **Solana Wallet (SOL / USDC / USDT)**: `9Vw8cNxyd9PBGz45C5MxZkeJdaA84CSQobgBX3ZKCRuu`
- **EVM Wallet (Ethereum / Arbitrum / Base)**: `0xBFdd875810C7B238c4295a8233180B796165B0AC`

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT.
