//! Validación estática del WGSL EPI4 (sin ejecutar simulación Bevy).

use std::path::Path;

use resonance::worldgen::CELL_FIELD_SNAPSHOT_WGSL_PATH;

fn load_cell_field_snapshot_wgsl() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CELL_FIELD_SNAPSHOT_WGSL_PATH);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("leer WGSL {:?}: {e}", path))
}

#[test]
fn cell_field_snapshot_wgsl_parses_with_naga() {
    let src = load_cell_field_snapshot_wgsl();
    let module = naga::front::wgsl::parse_str(&src).expect("WGSL EPI4 debe compilar con naga");
    assert!(
        !module.entry_points.is_empty(),
        "debe existir al menos un entry point (palette_dispatch)"
    );
}
