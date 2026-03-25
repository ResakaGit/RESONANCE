// Cuerpo de tests del módulo `equations` (incluido por `tests.rs`).
// Si se amplía la suite, preferir tests por dominio en `equations/<dominio>/` o regenerar desde el script de split histórico.

#[test]
fn smoke_sphere_volume_positive() {
    assert!(sphere_volume(1.0) > 0.0);
}

#[test]
fn smoke_density_finite_for_unit_sphere() {
    let d = density(100.0, 1.0);
    assert!(d.is_finite() && d > 0.0);
}

#[test]
fn smoke_interference_bounded() {
    let i = interference(440.0, 0.0, 440.0, 0.0, 0.25);
    assert!((-1.01..=1.01).contains(&i));
}

#[test]
fn smoke_metabolic_graph_empty_manifest() {
    use crate::layers::{LifecycleStage, OrganManifest};
    let m = OrganManifest::new(LifecycleStage::Growing);
    let g = metabolic_graph_from_manifest(&m, 300.0, 280.0);
    assert_eq!(g.node_count(), 0);
}

// ── SM-8C: extracted equation tests ──

#[test]
fn projectile_dissipation_positive_and_scales_with_radius() {
    let small = projectile_dissipation(0.1);
    let large = projectile_dissipation(5.0);
    assert!(small > 0.0);
    assert!(large > small);
    assert!((projectile_dissipation(0.0) - 0.5).abs() < 1e-6);
}

#[test]
fn allometric_max_radius_respects_factor_and_floor() {
    use crate::blueprint::constants::VOLUME_MIN_RADIUS;
    let r = allometric_max_radius(1.0, 3.0);
    assert!((r - 3.0).abs() < 1e-6);
    let tiny = allometric_max_radius(0.0, 3.0);
    assert!((tiny - VOLUME_MIN_RADIUS).abs() < 1e-6);
}

#[test]
fn normalized_qe_clamped_to_unit_interval() {
    assert!((normalized_qe(300.0, 600.0) - 0.5).abs() < 1e-6);
    assert!((normalized_qe(1000.0, 600.0) - 1.0).abs() < 1e-6);
    assert!((normalized_qe(0.0, 600.0)).abs() < 1e-6);
    assert!((normalized_qe(100.0, 0.0) - 1.0).abs() < 1e-6);
}
