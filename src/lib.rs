//! # S.P.E.C.U.L.A.M. Solver
//!
//! A zero-allocation flat constraint solver in Rust designed for ultra-low latency DeFi arbitrage
//! and MEV (Maximal Extractable Value) on high-performance blockchains like Solana.
//!
//! The solver is inspired by physical mechanical springs, modeling constraints as elastic/rigid structures
//! and minimizing global energy/stress using parallel lock-free gradient descent.

pub mod autogenesis;
pub mod espejo;
pub mod estres;
pub mod grafo;
pub mod network_bridge;
pub mod quantum_jump;

// Re-exportar elementos clave para facilitar el uso
pub use autogenesis::Autogenesis;
pub use espejo::{MotorSpeculam, SolucionEspejo};
pub use estres::CampoEstres;
pub use grafo::{Restriccion, SistemaRestricciones};
pub use network_bridge::NetworkBridge;
pub use quantum_jump::QuantumJumper;
