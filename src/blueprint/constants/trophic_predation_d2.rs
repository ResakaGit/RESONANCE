// ── D2: Trophic & Predation — constantes de cadena trófica ──

/// Eficiencia de transferencia trófica (Lindeman 10% rule).
pub const TROPHIC_TRANSFER_EFFICIENCY: f32 = 0.10;
/// Eficiencia de asimilación herbívoro (planta → herbívoro).
pub const HERBIVORE_ASSIMILATION: f32 = 0.35;
/// Eficiencia de asimilación carnívoro (carne → carnívoro).
pub const CARNIVORE_ASSIMILATION: f32 = 0.20;
/// Eficiencia de asimilación descomponedor (cadáver → descomponedor).
pub const DECOMPOSER_ASSIMILATION: f32 = 0.15;
/// Probabilidad base de éxito en caza (30%).
pub const PREDATION_BASE_SUCCESS: f32 = 0.3;
/// Escala de ventaja de velocidad del predador sobre la presa.
pub const PREDATION_SPEED_ADVANTAGE_SCALE: f32 = 0.5;
/// Máximo qe drenado por tick de una celda de nutrientes (foraging).
pub const FORAGING_CELL_DRAIN_MAX: f32 = 5.0;
/// Decay de saciedad por tick.
pub const SATIATION_DECAY_RATE: f32 = 0.005;
/// Ganancia de saciedad por alimentación exitosa.
pub const MEAL_SATIATION_GAIN: f32 = 0.3;
/// Presupuesto de queries espaciales por frame (throttle N²).
pub const TROPHIC_SCAN_BUDGET: usize = 64;
/// Radio de captura para predación (unidades mundo).
pub const PREDATION_CAPTURE_RADIUS: f32 = 3.0;
/// Umbral de saciedad bajo el cual se emite HungerEvent.
pub const HUNGER_THRESHOLD: f32 = 0.3;
/// Umbral de saciedad sobre el cual el predador no caza.
pub const PREDATION_WELL_FED_THRESHOLD: f32 = 0.8;
/// Piso de velocidad de presa (evita div/0).
pub const PREY_SPEED_FLOOR: f32 = 0.01;
/// Escala de penalización por distancia en predación.
pub const PREDATION_DISTANCE_PENALTY_SCALE: f32 = 0.05;
/// Penalización máxima por distancia en predación.
pub const PREDATION_DISTANCE_PENALTY_MAX: f32 = 0.5;
/// Escala de resistencia por bond_energy en transferencia de presa.
pub const PREY_BOND_RESISTANCE_SCALE: f32 = 0.01;
/// Factor de conversión intake_rate → velocidad de predador (proxy).
pub const PREDATION_INTAKE_TO_SPEED_FACTOR: f32 = 2.0;
/// Velocidad base de presa (neutral, sin movilidad especial).
pub const PREY_BASE_SPEED: f32 = 1.0;
/// Factor de terreno neutral (sin modificación).
pub const TERRAIN_FACTOR_NEUTRAL: f32 = 1.0;
/// Temperatura neutral (sin penalización térmica).
pub const TEMPERATURE_NEUTRAL: f32 = 0.5;
/// Escala de qe corporal devuelta a grid de nutrientes.
pub const DECOMPOSITION_GRID_RETURN_SCALE: f32 = 0.01;
/// Máximo delta de nutrientes por celda por descomposición.
pub const DECOMPOSITION_GRID_RETURN_MAX: f32 = 0.1;
/// Factor de ganancia de saciedad para descomponedores (relativo a MEAL_SATIATION_GAIN).
pub const DECOMPOSER_SATIATION_FACTOR: f32 = 0.5;
/// Epsilon para evitar div/0 en fracción de drenaje de celdas.
pub const CELL_QE_EPSILON: f32 = 0.001;
/// qe de referencia para cadáver cuando no se puede leer la energía real.
pub const DECOMPOSITION_DEFAULT_CORPSE_QE: f32 = 50.0;
