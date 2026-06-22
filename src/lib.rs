pub mod grafo;
pub mod estres;
pub mod espejo;
pub mod autogenesis;
pub mod quantum_jump;
pub mod network_bridge;

// Re-exportar elementos clave para facilitar el uso
pub use grafo::{SistemaRestricciones, Restriccion};
pub use estres::CampoEstres;
pub use espejo::{SolucionEspejo, MotorSpeculam};
pub use autogenesis::Autogenesis;
pub use quantum_jump::QuantumJumper;
pub use network_bridge::NetworkBridge;
