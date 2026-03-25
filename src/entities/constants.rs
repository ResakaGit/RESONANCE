//! Constantes de tuning para arquetipos (`entities/`).
//! EA2: presets de spawn Flora — una sola fuente de verdad para tests y `spawn_*`.

use crate::layers::CapabilitySet;

/// Símbolo RON (`ElementDef.symbol`) — mismo literal que `ABIOGENESIS_FLORA_ELEMENT_SYMBOL` (sin duplicar).
pub(crate) use crate::blueprint::constants::ABIOGENESIS_FLORA_ELEMENT_SYMBOL as FLORA_ELEMENT_SYMBOL;

/// Radio del icono minimapa para entidades flora spawn (paridad con semilla botánica).
pub(crate) const FLORA_MINIMAP_ICON_RADIUS: f32 = 12.0;

/// `GrowthBudget::limiter` en spawn inicial (sin limitante Liebig explícito).
pub(crate) const FLORA_GROWTH_LIMITER: u8 = 0;

/// Tabla fila EA2: parámetros de composición antes del insert inferencia/capabilities.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FloraSpawnPreset {
    pub entity_name: &'static str,
    pub minimap_rgb: [f32; 3],
    pub qe: f32,
    pub radius: f32,
    pub flow_dissipation: f32,
    pub bond_energy_eb: f32,
    pub thermal_conductivity: f32,
    pub nutrient_c: f32,
    pub nutrient_n: f32,
    pub nutrient_p: f32,
    pub nutrient_w: f32,
    pub growth_biomass: f32,
    pub growth_efficiency: f32,
    pub growth_bias: f32,
    pub mobility_bias: f32,
    pub branching_bias: f32,
    pub resilience: f32,
    pub capability_flags: u8,
}

pub(crate) mod flora_ea2 {
    //! Presets EA2 — `docs/sprints/ECOSYSTEM_AUTOPOIESIS/README.md` (track cerrado).

    use super::{CapabilitySet, FloraSpawnPreset};

    pub(crate) const ROSA: FloraSpawnPreset = FloraSpawnPreset {
        entity_name: "flora_rosa",
        minimap_rgb: [0.92, 0.35, 0.55],
        qe: 200.0,
        radius: 0.08,
        flow_dissipation: 0.005,
        bond_energy_eb: 800.0,
        thermal_conductivity: 0.05,
        nutrient_c: 30.0,
        nutrient_n: 20.0,
        nutrient_p: 15.0,
        nutrient_w: 50.0,
        growth_biomass: 0.08,
        growth_efficiency: 0.9,
        growth_bias: 0.9,
        mobility_bias: 0.0,
        branching_bias: 0.8,
        resilience: 0.5,
        capability_flags: CapabilitySet::GROW | CapabilitySet::BRANCH | CapabilitySet::ROOT,
    };

    pub(crate) const OAK: FloraSpawnPreset = FloraSpawnPreset {
        entity_name: "flora_oak",
        minimap_rgb: [0.22, 0.42, 0.18],
        qe: 300.0,
        radius: 0.12,
        flow_dissipation: 0.003,
        bond_energy_eb: 3000.0,
        thermal_conductivity: 0.3,
        nutrient_c: 50.0,
        nutrient_n: 30.0,
        nutrient_p: 30.0,
        nutrient_w: 40.0,
        growth_biomass: 0.04,
        growth_efficiency: 0.7,
        growth_bias: 0.6,
        mobility_bias: 0.0,
        branching_bias: 0.3,
        resilience: 0.9,
        capability_flags: CapabilitySet::GROW | CapabilitySet::BRANCH | CapabilitySet::ROOT,
    };

    pub(crate) const MOSS: FloraSpawnPreset = FloraSpawnPreset {
        entity_name: "flora_moss",
        minimap_rgb: [0.45, 0.72, 0.38],
        qe: 100.0,
        radius: 0.03,
        flow_dissipation: 0.008,
        bond_energy_eb: 200.0,
        thermal_conductivity: 0.02,
        nutrient_c: 20.0,
        nutrient_n: 15.0,
        nutrient_p: 10.0,
        nutrient_w: 60.0,
        growth_biomass: 0.12,
        growth_efficiency: 0.95,
        growth_bias: 1.0,
        mobility_bias: 0.0,
        branching_bias: 0.9,
        resilience: 0.2,
        capability_flags: CapabilitySet::GROW | CapabilitySet::BRANCH,
    };
}

// ── MG-8: Presets de arquetipos morfogenéticos ──────────────────────────────

/// Preset numérico para arquetipos de morfogénesis inferida (MG-8).
#[derive(Clone, Copy, Debug)]
pub(crate) struct MorphogenesisSpawnPreset {
    pub entity_name:      &'static str,
    pub qe:               f32,
    pub radius:           f32,
    pub velocity_x:       f32,
    pub dissipation:      f32,
    pub delta_qe:         f32,
    pub viscosity:        f32,
    pub photon_density:   f32,
    pub absorbed_fraction: f32,
    pub t_core_build:     f32,
    pub t_env_build:      f32,
    pub roles:            &'static [crate::layers::OrganRole],
}

pub(crate) mod morphogenesis_mg8 {
    use super::MorphogenesisSpawnPreset;
    use crate::layers::OrganRole;

    /// Organismo acuático: fusiforme, oscuro, liso.
    /// density(1500,0.8)≈699, T_env=310 → T_core >> T_env → dark albedo + smooth.
    pub(crate) const AQUATIC_ORGANISM: MorphogenesisSpawnPreset = MorphogenesisSpawnPreset {
        entity_name:      "aquatic_organism",
        qe:               1500.0,
        radius:           0.8,
        velocity_x:       15.0,
        dissipation:      0.05,
        delta_qe:         0.0,
        viscosity:        2.5,
        photon_density:   5.0,
        absorbed_fraction: 0.3,
        t_core_build:     699.0,
        t_env_build:      310.0,
        roles: &[OrganRole::Core, OrganRole::Stem, OrganRole::Fin, OrganRole::Fin, OrganRole::Sensory],
    };

    /// Planta desértica: compacta, clara, rugosa.
    /// density(200,1.0)≈48, T_env=284 → T_core << T_env → bright albedo + rough.
    pub(crate) const DESERT_PLANT: MorphogenesisSpawnPreset = MorphogenesisSpawnPreset {
        entity_name:      "desert_plant",
        qe:               200.0,
        radius:           1.0,
        velocity_x:       0.0,
        dissipation:      0.02,
        delta_qe:         -2.0,
        viscosity:        1.2,
        photon_density:   100.0,
        absorbed_fraction: 0.8,
        t_core_build:     48.0,
        t_env_build:      284.0,
        roles: &[OrganRole::Root, OrganRole::Stem, OrganRole::Leaf, OrganRole::Leaf, OrganRole::Thorn],
    };

    /// Criatura desértica: ligeramente alargada, clara, crestas.
    /// density(250,1.5)≈18, T_env=284 → T_core << T_env → bright.
    pub(crate) const DESERT_CREATURE: MorphogenesisSpawnPreset = MorphogenesisSpawnPreset {
        entity_name:      "desert_creature",
        qe:               250.0,
        radius:           1.5,
        velocity_x:       2.0,
        dissipation:      0.04,
        delta_qe:         -2.0,
        viscosity:        1.2,
        photon_density:   100.0,
        absorbed_fraction: 0.8,
        t_core_build:     18.0,
        t_env_build:      284.0,
        roles: &[OrganRole::Root, OrganRole::Core, OrganRole::Stem, OrganRole::Fin, OrganRole::Sensory],
    };

    /// Planta de bosque: forma intermedia, color medio.
    /// density(300,0.5)≈573, T_env=284 → T_core >> T_env.
    /// Irradiancia negligible → albedo = ALBEDO_FALLBACK (0.5).
    pub(crate) const FOREST_PLANT: MorphogenesisSpawnPreset = MorphogenesisSpawnPreset {
        entity_name:      "forest_plant",
        qe:               300.0,
        radius:           0.5,
        velocity_x:       0.0,
        dissipation:      0.03,
        delta_qe:         1.0,
        viscosity:        1.2,
        photon_density:   0.001,
        absorbed_fraction: 0.001,
        t_core_build:     573.0,
        t_env_build:      284.0,
        roles: &[OrganRole::Root, OrganRole::Stem, OrganRole::Leaf, OrganRole::Leaf, OrganRole::Leaf, OrganRole::Fruit],
    };
}
