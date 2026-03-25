use std::f32::consts::PI;

use bevy::prelude::*;

use crate::blueprint::{AlchemicalAlmanac, ElementId};
use crate::layers::OscillatorySignature;

/// Capa 2 (Resonancia): garantiza que toda entidad con `OscillatorySignature`
/// tenga un `ElementId` derivable desde el `AlchemicalAlmanac`.
///
/// - Si `ElementId` no existe: insertar.
/// - Si cambia la frecuencia: sincronizar el `ElementId` mapeado.
///
/// Trade-off: por ahora derivamos `ElementId` desde `frequency_hz`
/// (transición incremental hacia V3). En el próximo paso V3, `frequency_hz`
/// se derivará desde `ElementId`.
pub fn ensure_element_id_component_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    mut missing_query: Query<(Entity, &OscillatorySignature), Without<ElementId>>,
) {
    for (entity, signature) in &mut missing_query {
        if let Some(id) = almanac.find_stable_band_id(signature.frequency_hz()) {
            commands.entity(entity).insert(id);
        }
    }
}

/// Inyección V3 en Capa 2: deriva `OscillatorySignature.frequency_hz`
/// desde `ElementId` cuando el componente se agrega.
///
/// Importante: no debe disparar en cada cambio de `ElementId` (por ejemplo,
/// cuando la transmutación actualiza la frecuencia y luego sincronizamos el
/// `ElementId`). En ese caso, queremos preservar la `frequency_hz` mutada por gameplay.
pub fn derive_frequency_from_element_id_system(
    almanac: Res<AlchemicalAlmanac>,
    mut query: Query<(&ElementId, &mut OscillatorySignature), Added<ElementId>>,
) {
    for (element_id, mut signature) in &mut query {
        if let Some(def) = almanac.get(*element_id) {
            signature.set_frequency_hz(def.frequency_hz.max(0.0));
            // Normaliza la fase para mantener invariantes y evitar drift numérico.
            let ph = signature.phase().rem_euclid(2.0 * PI);
            signature.set_phase(ph);
        }
    }
}

/// Consistencia post-transmutación:
/// si `OscillatorySignature.frequency_hz` cambia (catálisis), sincronizamos `ElementId`
/// para que `perception_system` y `debug_gizmos_system` consulten el elemento correcto.
///
/// Trade-off: si la frecuencia cae fuera de cualquier banda estable, dejamos `ElementId`
/// como estaba; la `purity()` del `ElementDef` ya devuelve ~0 dentro de band boundaries,
/// por lo que la percepción cae igualmente.
pub fn sync_element_id_from_frequency_system(
    almanac: Res<AlchemicalAlmanac>,
    mut query: Query<(&OscillatorySignature, &mut ElementId), Changed<OscillatorySignature>>,
) {
    for (signature, mut element_id) in &mut query {
        if let Some(new_id) = almanac.find_stable_band_id(signature.frequency_hz()) {
            if element_id.0 != new_id.0 {
                element_id.0 = new_id.0;
            }
        }
    }
}
