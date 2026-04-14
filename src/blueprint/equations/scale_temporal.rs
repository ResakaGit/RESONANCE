//! Scheduler temporal adaptativo por escala — K, regímenes, ajuste cross-scale.
//! Per-scale adaptive temporal scheduler — K, regimes, cross-scale adjustment.
//!
//! CT-7 / ADR-036 §D5. Pure math.
//!
//! **Desacoplado de `batch::telescope::TelescopeStack`**: ese stack está atado a
//! `SimWorldFlat` (f32/2D). Esta abstracción provee solo el scheduler de K +
//! régimen — el equivalente funcional sobre `CosmicWorld` (f64/3D).

// ─── Scale telescope K ranges ───────────────────────────────────────────────

/// Rango de K admisible por escala. Fenómenos más lentos → K mayor.
/// Admissible K range per scale. Slower phenomena → larger K.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaleTelescopeBounds {
    pub k_min: u32,
    pub k_max: u32,
}

impl ScaleTelescopeBounds {
    pub const COSMOLOGICAL: Self = Self { k_min: 256, k_max: 16_384 };
    pub const STELLAR: Self      = Self { k_min: 64,  k_max: 4_096 };
    pub const PLANETARY: Self    = Self { k_min: 16,  k_max: 1_024 };
    pub const ECOLOGICAL: Self   = Self { k_min: 4,   k_max: 256 };
    pub const MOLECULAR: Self    = Self { k_min: 2,   k_max: 64 };

    /// Default para cada escala. Table from CT-7 §E2.
    /// Defaults per scale (CT-7 §E2).
    pub const fn for_depth(depth: u8) -> Self {
        match depth {
            0 => Self::COSMOLOGICAL,
            1 => Self::STELLAR,
            2 => Self::PLANETARY,
            3 => Self::ECOLOGICAL,
            _ => Self::MOLECULAR,
        }
    }

    /// Clamp un K dado a los bounds de esta escala.
    /// Clamp a given K to these bounds.
    #[inline]
    pub const fn clamp(&self, k: u32) -> u32 {
        if k < self.k_min { self.k_min }
        else if k > self.k_max { self.k_max }
        else { k }
    }
}

// ─── Regime hints ───────────────────────────────────────────────────────────

/// Clasificación del régimen dinámico de una escala.
/// Dynamic regime classification of a scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeHint {
    /// Evolución predecible. K puede subir.
    Stasis,
    /// Cambio notable pero acotado. K a mitad de rango.
    Transition,
    /// Lyapunov alto / caos. K debe bajar.
    Chaos,
}

/// Deriva un hint desde la relación `Δqe_norm` sobre una ventana reciente.
/// Derives a hint from `Δqe_norm` over a recent window.
///
/// `delta_qe_norm = |qe_now - qe_prev| / (qe_prev + ε)` — relativo, adimensional.
/// Thresholds: < 0.01 Stasis; < 0.1 Transition; ≥ 0.1 Chaos.
#[inline]
pub fn regime_from_qe_delta(delta_qe_norm: f64) -> RegimeHint {
    if delta_qe_norm < 0.01 { RegimeHint::Stasis }
    else if delta_qe_norm < 0.1 { RegimeHint::Transition }
    else { RegimeHint::Chaos }
}

// ─── Cross-scale K adjustment ───────────────────────────────────────────────

/// Ajusta el K de la escala hija según el régimen del padre.
/// Adjusts child K based on parent regime.
///
/// - `Stasis`:     K × 2 (padre estable → hijo puede proyectar lejos)
/// - `Transition`: K × 1 (sin cambio)
/// - `Chaos`:      K ÷ 2 (padre turbulento → hijo proyecta poco)
///
/// Clampado a los bounds del hijo.
pub fn adjust_k_from_parent(child_k: u32, child_bounds: ScaleTelescopeBounds, hint: RegimeHint) -> u32 {
    let raw = match hint {
        RegimeHint::Stasis => child_k.saturating_mul(2),
        RegimeHint::Transition => child_k,
        RegimeHint::Chaos => child_k / 2,
    };
    child_bounds.clamp(raw)
}

// ─── Scale telescope state ──────────────────────────────────────────────────

/// Estado temporal por instancia de escala. Reemplazo ligero de `TelescopeStack`
/// mientras ese tipo sigue atado a `SimWorldFlat`.
/// Per-scale temporal state. Lightweight replacement for `TelescopeStack`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaleTelescope {
    pub k: u32,
    pub bounds: ScaleTelescopeBounds,
    pub last_regime: RegimeHint,
}

impl ScaleTelescope {
    /// Construye con K = k_min (conservador) y régimen Transition (neutral).
    /// Builds with K = k_min (conservative) and Transition regime (neutral).
    pub const fn for_depth(depth: u8) -> Self {
        let bounds = ScaleTelescopeBounds::for_depth(depth);
        Self { k: bounds.k_min, bounds, last_regime: RegimeHint::Transition }
    }

    /// Aplica `adjust_k_from_parent` preservando bounds.
    /// Applies `adjust_k_from_parent` preserving bounds.
    pub fn adjust_from_parent(&mut self, parent_hint: RegimeHint) {
        let new_k = adjust_k_from_parent(self.k, self.bounds, parent_hint);
        if self.k != new_k { self.k = new_k; }
        if self.last_regime != parent_hint { self.last_regime = parent_hint; }
    }

    /// Actualiza K usando el propio régimen local (self-adaptive).
    /// Updates K using the local regime (self-adaptive).
    pub fn update_from_local_regime(&mut self, local: RegimeHint) {
        let target = match local {
            RegimeHint::Stasis => self.bounds.k_max,
            RegimeHint::Transition => (self.bounds.k_min + self.bounds.k_max) / 2,
            RegimeHint::Chaos => self.bounds.k_min,
        };
        if self.k != target { self.k = target; }
        if self.last_regime != local { self.last_regime = local; }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_match_ct7_table() {
        assert_eq!(ScaleTelescopeBounds::for_depth(0), ScaleTelescopeBounds::COSMOLOGICAL);
        assert_eq!(ScaleTelescopeBounds::for_depth(1), ScaleTelescopeBounds::STELLAR);
        assert_eq!(ScaleTelescopeBounds::for_depth(2), ScaleTelescopeBounds::PLANETARY);
        assert_eq!(ScaleTelescopeBounds::for_depth(3), ScaleTelescopeBounds::ECOLOGICAL);
        assert_eq!(ScaleTelescopeBounds::for_depth(4), ScaleTelescopeBounds::MOLECULAR);
    }

    #[test]
    fn cosmological_scale_allows_higher_k_than_molecular() {
        assert!(ScaleTelescopeBounds::COSMOLOGICAL.k_max > ScaleTelescopeBounds::MOLECULAR.k_max);
        assert!(ScaleTelescopeBounds::COSMOLOGICAL.k_min > ScaleTelescopeBounds::MOLECULAR.k_min);
    }

    #[test]
    fn bounds_clamp_within_range() {
        let b = ScaleTelescopeBounds::ECOLOGICAL;
        assert_eq!(b.clamp(0), b.k_min);
        assert_eq!(b.clamp(u32::MAX), b.k_max);
        assert_eq!(b.clamp(b.k_min), b.k_min);
        assert_eq!(b.clamp(b.k_max), b.k_max);
        let mid = (b.k_min + b.k_max) / 2;
        assert_eq!(b.clamp(mid), mid);
    }

    #[test]
    fn regime_from_qe_delta_thresholds() {
        assert_eq!(regime_from_qe_delta(0.0), RegimeHint::Stasis);
        assert_eq!(regime_from_qe_delta(0.005), RegimeHint::Stasis);
        assert_eq!(regime_from_qe_delta(0.05), RegimeHint::Transition);
        assert_eq!(regime_from_qe_delta(0.2), RegimeHint::Chaos);
    }

    #[test]
    fn parent_stasis_doubles_child_k() {
        let b = ScaleTelescopeBounds::ECOLOGICAL;
        let k = 10;
        assert_eq!(adjust_k_from_parent(k, b, RegimeHint::Stasis), b.clamp(20));
    }

    #[test]
    fn parent_chaos_halves_child_k() {
        let b = ScaleTelescopeBounds::ECOLOGICAL;
        let k = 100;
        assert_eq!(adjust_k_from_parent(k, b, RegimeHint::Chaos), b.clamp(50));
    }

    #[test]
    fn parent_transition_keeps_child_k() {
        let b = ScaleTelescopeBounds::PLANETARY;
        let k = 32;
        assert_eq!(adjust_k_from_parent(k, b, RegimeHint::Transition), 32);
    }

    #[test]
    fn adjust_respects_bounds() {
        let b = ScaleTelescopeBounds::MOLECULAR; // k_min=2, k_max=64
        // Stasis would double 40 to 80, but clamp to 64.
        assert_eq!(adjust_k_from_parent(40, b, RegimeHint::Stasis), 64);
        // Chaos halves 3 to 1, but clamp to 2.
        assert_eq!(adjust_k_from_parent(3, b, RegimeHint::Chaos), 2);
    }

    #[test]
    fn scale_telescope_for_depth_starts_conservative() {
        let t = ScaleTelescope::for_depth(0);
        assert_eq!(t.k, ScaleTelescopeBounds::COSMOLOGICAL.k_min);
        assert_eq!(t.last_regime, RegimeHint::Transition);
    }

    #[test]
    fn scale_telescope_adjust_from_parent_persists_state() {
        let mut t = ScaleTelescope::for_depth(3); // Ecological
        let initial_k = t.k;
        t.adjust_from_parent(RegimeHint::Stasis);
        assert!(t.k >= initial_k);
        assert_eq!(t.last_regime, RegimeHint::Stasis);
    }

    #[test]
    fn scale_telescope_update_from_local_regime_targets_bounds() {
        let mut t = ScaleTelescope::for_depth(2); // Planetary
        t.update_from_local_regime(RegimeHint::Chaos);
        assert_eq!(t.k, ScaleTelescopeBounds::PLANETARY.k_min);
        t.update_from_local_regime(RegimeHint::Stasis);
        assert_eq!(t.k, ScaleTelescopeBounds::PLANETARY.k_max);
    }

    #[test]
    fn molecular_scale_bounded_tightly_for_lyapunov() {
        let b = ScaleTelescopeBounds::MOLECULAR;
        assert!(b.k_max <= 64, "molecular K too high for chaotic MD");
    }

    #[test]
    fn cosmological_scale_allows_large_projections() {
        let b = ScaleTelescopeBounds::COSMOLOGICAL;
        assert!(b.k_max >= 16_384);
    }
}
