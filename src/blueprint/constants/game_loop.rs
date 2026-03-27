/// qe mínimo bajo el cual un núcleo es considerado destruido.
pub const QE_NUCLEUS_VIABILITY_THRESHOLD: f32 = 100.0;

/// Intake base del núcleo en qe/tick antes de modificadores de daño.
pub const NUCLEUS_BASE_INTAKE_QE_PER_TICK: f32 = 50.0;

/// Escalamiento de snowball: por cada qe de ventaja, incremento de intake.
pub const SNOWBALL_INTAKE_SCALING: f32 = 0.01;
