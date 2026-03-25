// ── D6: Social & Communication — constantes de manada ──

/// Fuerza de cohesión hacia el centroide del pack.
pub const COHESION_STRENGTH: f32 = 0.3;
/// Peso de resiliencia en score de dominancia.
pub const DOMINANCE_RESILIENCE_WEIGHT: f32 = 0.5;
/// Escala del bonus cooperativo de caza (√N).
pub const COOPERATIVE_HUNT_SCALE: f32 = 1.0;
/// Radio dentro del cual entidades de misma facción forman pack.
pub const PACK_FORMATION_RADIUS: f32 = 8.0;
/// Stiffness del enlace social (StructuralLink blando).
pub const SOCIAL_BOND_STIFFNESS: f32 = 0.01;
/// Rest length del enlace social (distancia de grupo).
pub const SOCIAL_BOND_REST_LENGTH: f32 = 7.0;
/// Break stress del enlace social (se rompe si se separan mucho).
pub const SOCIAL_BOND_BREAK_STRESS: f32 = 50.0;
/// Intervalo de ticks entre intentos de formación de pack.
pub const PACK_FORMATION_TICK_INTERVAL: u64 = 16;
/// Intervalo de ticks entre recálculos de dominancia.
pub const DOMINANCE_TICK_INTERVAL: u64 = 60;
/// Presupuesto de queries espaciales para formación de pack por frame.
pub const SOCIAL_SCAN_BUDGET: usize = 32;
