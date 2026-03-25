// EPI4 — Snapshot celular (SSBO) + `quantized_palette_index` alineado a `blueprint::equations::quantized_palette_index`.
// Si cambia la fórmula en Rust, actualizar aquí y `tests/gpu_cell_field_snapshot_palette_dispatch.rs`.
// `CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION` y ruta `CELL_FIELD_SNAPSHOT_WGSL_PATH` en `gpu_layout.rs`.

struct GpuCellFieldSnapshotHeader {
    snapshot_schema_version: u32,
    grid_width: u32,
    grid_height: u32,
    grid_generation: u32,
}

struct GpuCellFieldPacked {
    accumulated_qe: f32,
    dominant_frequency_hz: f32,
    purity: f32,
    temperature: f32,
    matter_state: u32,
    materialized_present: u32,
    materialized_index: u32,
    materialized_generation: u32,
    contributions_fingerprint: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

const QUANTIZED_COLOR_RHO_CLAMP_FLOOR: f32 = 0.000001;

// NaN IEEE (exponent 0xFF, mantisa ≠ 0); inf (exponent 0xFF, mantisa 0). Alineado a `equations::quantized_palette_index`.
fn ieee_is_nan(x: f32) -> bool {
    let u = bitcast<u32>(x);
    let exp = (u >> 23u) & 255u;
    let frac = u & 0x7fffffu;
    return exp == 255u && frac != 0u;
}

fn ieee_is_inf(x: f32) -> bool {
    let u = bitcast<u32>(x);
    let exp = (u >> 23u) & 255u;
    let frac = u & 0x7fffffu;
    return exp == 255u && frac == 0u;
}

fn canonical_enorm(enorm: f32) -> f32 {
    if ieee_is_nan(enorm) {
        return 0.0;
    }
    if ieee_is_inf(enorm) {
        if enorm < 0.0 {
            return 0.0;
        }
        return 1.0;
    }
    if enorm < -1e38 {
        return 0.0;
    }
    if enorm > 1e38 {
        return 1.0;
    }
    return clamp(enorm, 0.0, 1.0);
}

fn canonical_rho(rho: f32) -> f32 {
    if ieee_is_nan(rho) || ieee_is_inf(rho) {
        return 1.0;
    }
    if rho > 1e38 || rho < -1e38 {
        return 1.0;
    }
    return clamp(rho, QUANTIZED_COLOR_RHO_CLAMP_FLOOR, 1.0);
}

fn quantized_palette_index(enorm: f32, rho: f32, n_max: u32) -> u32 {
    if n_max <= 1u {
        return 0u;
    }
    let n_max_f = f32(n_max);
    let e = canonical_enorm(enorm);
    let r = canonical_rho(rho);
    let s = max(ceil(n_max_f * r), 1.0);
    let eq = floor(e * s) / s;
    var idx_f = floor(eq * (n_max_f - 1.0));
    if ieee_is_nan(idx_f) || ieee_is_inf(idx_f) || idx_f > 1e38 || idx_f < -1e38 {
        idx_f = 0.0;
    }
    let idx = u32(idx_f);
    return min(idx, n_max - 1u);
}

// --- Test compute (dispatch 1×1×1): `vec4<u32>` evita ambigüedad de layout struct en storage.
// x = bitcast(enorm), y = bitcast(rho), z = n_max, w = 0
@group(0) @binding(0) var<storage, read> palette_dispatch_words: vec4<u32>;
@group(0) @binding(1) var<storage, read_write> palette_dispatch_out: array<u32, 1u>;

@compute @workgroup_size(1u, 1u, 1u)
fn palette_dispatch() {
    let w = palette_dispatch_words;
    let enorm = bitcast<f32>(w.x);
    let rho = bitcast<f32>(w.y);
    let n_max = w.z;
    palette_dispatch_out[0] = quantized_palette_index(enorm, rho, n_max);
}
