use crate::blueprint::constants::*;

/// Permeabilidad osmótica entre dos celdas según diferencial de electronegatividad.
#[inline]
pub fn osmotic_permeability(electronegativity_a: f32, electronegativity_b: f32) -> f32 {
    let ea = if electronegativity_a.is_finite() {
        electronegativity_a.max(0.0)
    } else {
        0.0
    };
    let eb = if electronegativity_b.is_finite() {
        electronegativity_b.max(0.0)
    } else {
        0.0
    };
    OSMOTIC_BASE_PERMEABILITY * (1.0 + (ea - eb).abs() * OSMOTIC_ELECTRO_SCALE)
}

/// Delta de presión osmótica (A → B): positivo implica flujo neto de A hacia B.
#[inline]
pub fn osmotic_pressure_delta(
    concentration_a: f32,
    concentration_b: f32,
    membrane_permeability: f32,
) -> f32 {
    let ca = if concentration_a.is_finite() {
        concentration_a.max(0.0)
    } else {
        0.0
    };
    let cb = if concentration_b.is_finite() {
        concentration_b.max(0.0)
    } else {
        0.0
    };
    let p = if membrane_permeability.is_finite() {
        membrane_permeability.max(0.0)
    } else {
        0.0
    };
    (ca - cb) * p
}

/// Concentración osmótica desde energía de celda y volumen.
#[inline]
pub fn osmotic_concentration(cell_qe: f32, cell_volume: f32) -> f32 {
    if !cell_qe.is_finite() || !cell_volume.is_finite() || cell_volume <= 0.0 {
        return 0.0;
    }
    cell_qe.max(0.0) / cell_volume
}

/// Mezcla de frecuencia dominante al transferir energía entre dos celdas.
#[inline]
pub fn osmotic_frequency_mix(
    freq_src: f32,
    qe_src: f32,
    freq_dst: f32,
    qe_dst: f32,
    moved: f32,
) -> (f32, f32) {
    if moved <= 0.0 {
        return (freq_src, freq_dst);
    }
    let src_mass = qe_src.max(0.0);
    let dst_mass = qe_dst.max(0.0);
    let moved = moved.min(src_mass);
    let src_hz = if freq_src.is_finite() {
        freq_src.max(0.0)
    } else {
        0.0
    };
    let dst_hz = if freq_dst.is_finite() {
        freq_dst.max(0.0)
    } else {
        0.0
    };

    let src_after = (src_mass - moved).max(0.0);
    let dst_after = dst_mass + moved;
    let dst_new_hz = if dst_after > 0.0 {
        (dst_hz * dst_mass + src_hz * moved) / dst_after
    } else {
        0.0
    };
    let src_new_hz = if src_after > 0.0 { src_hz } else { 0.0 };
    (src_new_hz, dst_new_hz)
}

/// Escala de depleción de nutrientes por tick (Capa 4).
#[inline]
pub fn nutrient_depletion_scale(qe: f32, depletion_rate: f32) -> f32 {
    let q = if qe.is_finite() { qe.max(0.0) } else { 0.0 };
    let rate = if depletion_rate.is_finite() {
        depletion_rate.max(0.0)
    } else {
        0.0
    };
    (q * rate).clamp(0.0, 1.0)
}

/// Escala de retorno de nutrientes al morir una entidad.
#[inline]
pub fn nutrient_return_scale(qe: f32, return_rate: f32) -> f32 {
    let q = if qe.is_finite() { qe.max(0.0) } else { 0.0 };
    let rate = if return_rate.is_finite() {
        return_rate.max(0.0)
    } else {
        0.0
    };
    (q * rate).clamp(0.0, 1.0)
}

/// Eficiencia genética de crecimiento desde propiedades del elemento.
#[inline]
pub fn genetic_efficiency_for_element(bond_energy: f32, electronegativity: f32) -> f32 {
    let be = if bond_energy.is_finite() {
        bond_energy.max(0.0)
    } else {
        BOND_ENERGY_REFERENCE
    };
    let en = if electronegativity.is_finite() {
        electronegativity.max(0.0)
    } else {
        0.0
    };
    let flexibility = (1.0 - be / BOND_ENERGY_REFERENCE).clamp(0.1, 1.0);
    (flexibility * (1.0 + en * GENETIC_ELECTRO_BONUS)).clamp(0.1, 1.0)
}

/// Ley del mínimo de Liebig: el recurso más escaso limita la biomasa.
#[inline]
pub fn liebig_growth_budget(
    carbon: f32,
    nitrogen: f32,
    phosphorus: f32,
    water: f32,
    genetic_efficiency: f32,
) -> (f32, u8) {
    let c = if carbon.is_finite() {
        carbon.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let n = if nitrogen.is_finite() {
        nitrogen.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let p = if phosphorus.is_finite() {
        phosphorus.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let w = if water.is_finite() {
        water.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let eff = if genetic_efficiency.is_finite() {
        genetic_efficiency.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut limiter = 0u8;
    let mut min_val = c;
    if n < min_val {
        min_val = n;
        limiter = 1;
    }
    if p < min_val {
        min_val = p;
        limiter = 2;
    }
    if w < min_val {
        min_val = w;
        limiter = 3;
    }
    (min_val * eff, limiter)
}

/// Eficiencia gaussiana normalizada.
/// Retorna 1.0 en `value == optimal`, decae simétricamente al alejarse.
#[inline]
pub fn gaussian_efficiency(value: f32, optimal: f32, sigma: f32) -> f32 {
    if !value.is_finite() || !optimal.is_finite() || !sigma.is_finite() || sigma <= 0.0 {
        return 0.0;
    }
    let z = (value - optimal) / sigma;
    (-0.5 * z * z).exp().clamp(0.0, 1.0)
}

/// Rendimiento fotosintético (Capa 4): irradiancia × limitante hídrico/carbono × eficiencia térmica.
#[inline]
pub fn photosynthetic_yield(
    photon_density: f32,
    water_norm: f32,
    carbon_norm: f32,
    temperature_norm: f32,
) -> f32 {
    let photon = if photon_density.is_finite() {
        photon_density.max(0.0)
    } else {
        0.0
    };
    let water = if water_norm.is_finite() {
        water_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let carbon = if carbon_norm.is_finite() {
        carbon_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let temp = if temperature_norm.is_finite() {
        temperature_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let limiting = water.min(carbon);
    let raw = photon * limiting;
    let temp_eff = gaussian_efficiency(temp, PHOTO_OPTIMAL_TEMP_NORM, PHOTO_TEMP_SIGMA);
    raw * temp_eff
}

/// Atenuación radial de irradiancia emitida por una fuente.
#[inline]
pub fn irradiance_at_distance(source_emission: f32, distance: f32, decay: f32) -> f32 {
    let emission = if source_emission.is_finite() {
        source_emission.max(0.0)
    } else {
        0.0
    };
    let d = if distance.is_finite() {
        distance.max(0.0)
    } else {
        0.0
    };
    let k = if decay.is_finite() {
        decay.max(0.0)
    } else {
        0.0
    };
    emission / (1.0 + d * d * k)
}

/// Atenuación radial de irradiancia usando distancia al cuadrado (hot-path friendly).
#[inline]
pub fn irradiance_at_distance_sq(source_emission: f32, distance_sq: f32, decay: f32) -> f32 {
    let emission = if source_emission.is_finite() {
        source_emission.max(0.0)
    } else {
        0.0
    };
    let d2 = if distance_sq.is_finite() {
        distance_sq.max(0.0)
    } else {
        0.0
    };
    let k = if decay.is_finite() {
        decay.max(0.0)
    } else {
        0.0
    };
    emission / (1.0 + d2 * k)
}

/// Bonus fotosintético para growth budget, acotado para no anular Liebig.
#[inline]
pub fn photosynthetic_growth_bonus(photon_density: f32, absorbed_fraction: f32) -> f32 {
    let photon = if photon_density.is_finite() {
        photon_density.clamp(0.0, PHOTO_MAX_PHOTON_DENSITY)
    } else {
        0.0
    };
    let absorbed = if absorbed_fraction.is_finite() {
        absorbed_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (photon * absorbed * PHOTO_GROWTH_BONUS).clamp(0.0, PHOTO_GROWTH_BONUS_CAP)
}

