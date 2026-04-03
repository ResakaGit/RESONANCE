// ── EPI3 — GF1: modulación de tinte por rol de rama (tabla de datos; sin especie en hot path) ──
/// Mezcla en \[0, 1\] hacia acento STEM (tronco / madera) en RGB lineal.
pub const GF1_BRANCH_ROLE_BLEND_STEM: f32 = 0.08;
/// Mezcla hacia acento LEAF.
pub const GF1_BRANCH_ROLE_BLEND_LEAF: f32 = 0.28;
/// Mezcla hacia acento THORN.
pub const GF1_BRANCH_ROLE_BLEND_THORN: f32 = 0.22;
/// Acento RGB lineal STEM (referencia de diseño).
pub const GF1_BRANCH_ROLE_ACCENT_STEM_LIN: [f32; 3] = [0.52, 0.38, 0.24];
/// Acento RGB lineal LEAF.
pub const GF1_BRANCH_ROLE_ACCENT_LEAF_LIN: [f32; 3] = [0.18, 0.82, 0.22];
/// Acento RGB lineal THORN.
pub const GF1_BRANCH_ROLE_ACCENT_THORN_LIN: [f32; 3] = [0.45, 0.38, 0.32];
