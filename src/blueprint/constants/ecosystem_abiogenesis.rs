// ── Ecosistema: abiogénesis (EA5) — ecuaciones + calibración compartida con `simulation::abiogenesis` ──
/// Símbolo RON (`ElementDef.symbol`) para `ElementId::from_name` — **no** el campo `name` (display).
/// Debe coincidir con `assets/elements/flora.ron` (EAC1: RON manda identidad).
pub const ABIOGENESIS_FLORA_ELEMENT_SYMBOL: &str = "Fl";
/// Banda Flora (Hz) para gating de abiogenesis (legacy; axiomatic replaces); debe coincidir con `freq_band` en `flora.ron`.
pub const ABIOGENESIS_FLORA_BAND_HZ_LOW: f32 = 85.0;
pub const ABIOGENESIS_FLORA_BAND_HZ_HIGH: f32 = 110.0;
/// Pico de resonancia Flora (Hz) para el factor de proximidad en abiogenesis (legacy; axiomatic replaces); `frequency_hz` en `flora.ron`.
pub const ABIOGENESIS_FLORA_PEAK_HZ: f32 = 85.0;
/// Épsilon: en `abiogenesis_frequency_proximity`, pico tratado como coincidente con borde `low` / `high`.
pub const ABIOGENESIS_FREQ_TRIANGLE_EDGE_EPS: f32 = f32::EPSILON;
/// qe mínimo en celda para que abiogenesis (legacy; axiomatic replaces) pueda ser > 0.
pub const ABIOGENESIS_FIELD_MIN_QE: f32 = 30.0;
/// Umbral de score \[0, 1\] en `abiogenesis_system` para intentar spawn.
pub const ABIOGENESIS_POTENTIAL_SCORE_THRESHOLD: f32 = 0.6;

// ── Fixtures compartidos (tests abiogénesis / ecuaciones) ──
/// `cell_qe = ABIOGENESIS_FIELD_MIN_QE * este factor` en tests de spawn (supera cómodo el umbral).
pub const ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN: f32 = 3.0;
/// Saturación hídrica de celda en fixtures de grid de abiogénesis.
pub const ABIOGENESIS_TEST_FIXTURE_WATER_NORM: f32 = 0.8;
/// Tolerancia en tests de potencial ~0 en bordes del triángulo de frecuencia.
pub const ABIOGENESIS_TEST_POTENTIAL_NEAR_ZERO: f32 = 1e-5;
/// Tope de ratio qe/min_qe en el factor energético del potencial.
pub const ABIOGENESIS_POTENTIAL_QE_RATIO_CAP: f32 = 2.0;
/// Escala del factor energético (después del clamp del ratio).
pub const ABIOGENESIS_POTENTIAL_QE_RATIO_SCALE: f32 = 0.5;
/// Escala heurística: densidad de qe en celda → “bond local” para perfil y materia.
pub const ABIOGENESIS_CELL_QE_TO_BOND_SCALE: f32 = 10.0;
/// Clamp de `bond_energy_eb` al spawnear materia coherente.
pub const ABIOGENESIS_SPAWN_BOND_MIN: f32 = 200.0;
pub const ABIOGENESIS_SPAWN_BOND_MAX: f32 = 3000.0;
/// Fracción de qe celda que pasa a `BaseEnergy` del emergente.
pub const ABIOGENESIS_SPAWN_CELL_QE_FRACTION: f32 = 0.5;
/// Umbral de enlace heurístico: por encima → perfil “oak”.
pub const ABIOGENESIS_PROFILE_BOND_OAK_MIN: f32 = 2000.0;
/// Umbral de enlace heurístico: por debajo (con agua) → perfil “moss”.
pub const ABIOGENESIS_PROFILE_BOND_MOSS_MAX: f32 = 500.0;
/// Saturación hídrica mínima para rama “moss”.
pub const ABIOGENESIS_PROFILE_WATER_MOSS_MIN: f32 = 0.7;
/// Sesgos `(growth, branching, resilience)` por perfil emergente.
pub const ABIOGENESIS_OAK_GROWTH: f32 = 0.6;
pub const ABIOGENESIS_OAK_BRANCHING: f32 = 0.3;
pub const ABIOGENESIS_OAK_RESILIENCE: f32 = 0.9;
pub const ABIOGENESIS_MOSS_GROWTH: f32 = 1.0;
pub const ABIOGENESIS_MOSS_BRANCHING: f32 = 0.9;
pub const ABIOGENESIS_MOSS_RESILIENCE: f32 = 0.2;
pub const ABIOGENESIS_ROSA_GROWTH: f32 = 0.9;
pub const ABIOGENESIS_ROSA_BRANCHING: f32 = 0.8;
pub const ABIOGENESIS_ROSA_RESILIENCE: f32 = 0.5;

// ── Fauna Abiogenesis ───────────────────────────────────────────────────────

/// Minimum qe in cell for fauna spawn consideration (higher than flora — animals are costlier).
pub const ABIOGENESIS_FAUNA_FIELD_MIN_QE: f32 = 60.0;
/// Potential score threshold for fauna spawn [0, 1].
pub const ABIOGENESIS_FAUNA_POTENTIAL_THRESHOLD: f32 = 0.65;
/// Minimum flora occupants in 3×3 neighbourhood to permit herbivore spawn.
pub const ABIOGENESIS_FAUNA_MIN_FLORA_NEIGHBOURS: u32 = 3;
/// Minimum herbivore occupants in 5×5 neighbourhood to permit carnivore spawn.
pub const ABIOGENESIS_FAUNA_MIN_HERBIVORE_NEIGHBOURS: u32 = 2;
/// Nutrient water saturation floor for fauna spawn.
pub const ABIOGENESIS_FAUNA_WATER_FLOOR: f32 = 0.4;
/// Minimum average nutrient density [0, 1] for fauna spawn.
pub const ABIOGENESIS_FAUNA_NUTRIENT_DENSITY_MIN: f32 = 0.3;
/// Maximum fauna spawns per frame.
pub const ABIOGENESIS_FAUNA_MAX_PER_FRAME: usize = 1;
/// Fraction of cell qe that passes to the fauna entity.
pub const ABIOGENESIS_FAUNA_SPAWN_QE_FRACTION: f32 = 0.6;
/// Bond energy range for fauna spawn.
pub const ABIOGENESIS_FAUNA_SPAWN_BOND_MIN: f32 = 500.0;
pub const ABIOGENESIS_FAUNA_SPAWN_BOND_MAX: f32 = 2500.0;
/// Cell qe → bond heuristic scale for fauna.
pub const ABIOGENESIS_FAUNA_CELL_QE_TO_BOND_SCALE: f32 = 8.0;

// ── Fauna InferenceProfile biases ───────────────────────────────────────────
pub const ABIOGENESIS_HERBIVORE_GROWTH: f32 = 0.4;
pub const ABIOGENESIS_HERBIVORE_MOBILITY: f32 = 0.7;
pub const ABIOGENESIS_HERBIVORE_BRANCHING: f32 = 0.1;
pub const ABIOGENESIS_HERBIVORE_RESILIENCE: f32 = 0.5;
pub const ABIOGENESIS_CARNIVORE_GROWTH: f32 = 0.3;
pub const ABIOGENESIS_CARNIVORE_MOBILITY: f32 = 0.9;
pub const ABIOGENESIS_CARNIVORE_BRANCHING: f32 = 0.0;
pub const ABIOGENESIS_CARNIVORE_RESILIENCE: f32 = 0.7;

/// Herbivore intake rate (from TrophicConsumer).
pub const ABIOGENESIS_HERBIVORE_INTAKE_RATE: f32 = 12.0;
/// Carnivore intake rate.
pub const ABIOGENESIS_CARNIVORE_INTAKE_RATE: f32 = 18.0;
/// Initial satiation for newly spawned fauna.
pub const ABIOGENESIS_FAUNA_INITIAL_SATIATION: f32 = 0.5;
