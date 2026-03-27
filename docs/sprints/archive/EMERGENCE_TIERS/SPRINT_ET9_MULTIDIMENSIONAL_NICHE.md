# Sprint ET-9 — Multidimensional Niche: Hipervolumen de Hutchinson

**Módulo:** `src/layers/niche.rs` (nuevo), `src/blueprint/equations/emergence/niche.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T2-5. **Onda:** A.
**BridgeKind:** `NicheOverlapBridge` — cache Small(64), clave `hash(niche_a_band, niche_b_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

El nicho ecológico es un hipervolumen en el espacio de recursos: las dimensiones son frecuencia, posición, sustrato, tiempo. Dos entidades compiten cuando sus hipervolúmenes se solapan. La exclusión competitiva emerge: nichos idénticos no coexisten — uno excluye al otro o se diferencian.

```
niche(i) ⊂ ℝ⁴  (freq, spatial_x, spatial_z, temporal_phase)
overlap(i,j) = Vol(niche(i) ∩ niche(j)) / min(Vol(niche(i)), Vol(niche(j)))
competition(i,j) = overlap(i,j) × resource_demand_similarity
```

---

## Responsabilidades

### ET-9A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/niche.rs

/// Solapamiento de nicho entre dos entidades: [0,1].
/// niche_a/b: centro del nicho en 4D. width_a/b: radio por dimensión.
pub fn niche_overlap(
    niche_a: [f32; 4], width_a: [f32; 4],
    niche_b: [f32; 4], width_b: [f32; 4],
) -> f32 {
    let mut overlap_product = 1.0f32;
    for d in 0..4 {
        let dist = (niche_a[d] - niche_b[d]).abs();
        let combined_width = width_a[d] + width_b[d];
        if combined_width <= 0.0 { return 0.0; }
        let dim_overlap = (1.0 - dist / combined_width).clamp(0.0, 1.0);
        overlap_product *= dim_overlap;
    }
    overlap_product
}

/// Presión competitiva: solapamiento × demanda de recursos compartidos.
pub fn competitive_pressure(overlap: f32, resource_demand_a: f32, resource_demand_b: f32) -> f32 {
    overlap * (resource_demand_a * resource_demand_b).sqrt()
}

/// Desplazamiento de carácter: ajuste de nicho para reducir solapamiento.
/// Retorna el delta de centro hacia donde moverse para alejarse del competidor.
pub fn character_displacement(
    own_center: f32, competitor_center: f32, displacement_rate: f32,
) -> f32 {
    let direction = if own_center >= competitor_center { 1.0 } else { -1.0 };
    direction * displacement_rate
}

/// Amplitud del nicho: media geométrica de los radios en 4D.
pub fn niche_breadth(width: [f32; 4]) -> f32 {
    (width[0] * width[1] * width[2] * width[3]).powf(0.25)
}

/// Especialización óptima: nicho estrecho en entornos estables, amplio en variables.
pub fn optimal_niche_width(env_variance: f32, resource_density: f32) -> f32 {
    (env_variance / (resource_density + f32::EPSILON)).sqrt().clamp(0.1, 5.0)
}
```

### ET-9B: Componente

```rust
// src/layers/niche.rs

/// Capa T2-5: NicheProfile — hipervolumen de Hutchinson en 4D.
/// Dim 0: frecuencia preferida. Dim 1: x espacial. Dim 2: z espacial. Dim 3: fase temporal.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct NicheProfile {
    pub center:       [f32; 4],  // centro del nicho por dimensión
    pub width:        [f32; 4],  // radio por dimensión (± anchura del nicho)
    pub displacement_rate: f32,  // velocidad de desplazamiento de carácter
    pub specialization:   f32,   // [0,1] — 0=generalista, 1=especialista
}
```

### ET-9C: Sistema

```rust
/// Desplaza nichos de entidades en competición directa (character displacement).
/// Phase::MorphologicalLayer — modifica NicheProfile antes del siguiente ciclo.
pub fn niche_displacement_system(
    mut agents: Query<(Entity, &Transform, &mut NicheProfile, &AlchemicalEngine)>,
    spatial: Res<SpatialIndex>,
    mut cache: ResMut<BridgeCache<NicheOverlapBridge>>,
    config: Res<NicheConfig>,
    clock: Res<SimulationClock>,
) {
    if clock.tick_id % config.eval_interval as u64 != 0 { return; }

    // Collect snapshot to avoid aliasing
    let snapshot: Vec<(Entity, [f32; 4], [f32; 4], f32)> = agents.iter()
        .map(|(e, t, np, eng)| (e, np.center, np.width, eng.base_intake()))
        .collect();

    for (entity, transform, mut niche, engine) in &mut agents {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let nearby = spatial.query_radius(pos, config.competition_radius);

        for entry in &nearby {
            let competitor_entity = entry.entity;
            if competitor_entity == entity { continue; }
            let Some(&(_, c_center, c_width, c_demand)) = snapshot.iter()
                .find(|(e, _, _, _)| *e == competitor_entity) else { continue };

            // Cache key: hash de bandas de nicho de ambas entidades
            let key_a = niche_band_hash(niche.center, niche.width);
            let key_b = niche_band_hash(c_center, c_width);
            let cache_key = key_a ^ key_b.rotate_right(16);

            let overlap = if let Some(cached) = cache.get(cache_key) {
                cached
            } else {
                let ov = niche_eq::niche_overlap(niche.center, niche.width, c_center, c_width);
                cache.insert(cache_key, ov);
                ov
            };

            if overlap < config.overlap_threshold { continue; }

            // Desplazamiento de carácter: moverse en la dimensión de mayor solapamiento
            for d in 0..4 {
                let delta = niche_eq::character_displacement(
                    niche.center[d], c_center[d], niche.displacement_rate,
                );
                let new_center_d = niche.center[d] + delta * overlap;
                if (niche.center[d] - new_center_d).abs() > f32::EPSILON {
                    niche.center[d] = new_center_d;
                }
            }
        }
    }
}

fn niche_band_hash(center: [f32; 4], width: [f32; 4]) -> u32 {
    let mut h = 0u32;
    for i in 0..4 {
        h ^= ((center[i] * 10.0) as u32).rotate_left(i as u32 * 8);
        h ^= ((width[i]  * 10.0) as u32).rotate_left(i as u32 * 4 + 16);
    }
    h
}
```

### ET-9D: Constantes

```rust
pub struct NicheOverlapBridge;
impl BridgeKind for NicheOverlapBridge {}

pub const NICHE_DEFAULT_DISPLACEMENT_RATE: f32 = 0.01;  // lento — semanas en ticks
pub const NICHE_DEFAULT_OVERLAP_THRESHOLD: f32 = 0.3;   // sólo desplazar si >30% solapamiento
pub const NICHE_COMPETITION_RADIUS: f32 = 15.0;
pub const NICHE_EVAL_INTERVAL: u8 = 20;                  // re-evalúa cada 20 ticks
```

---

## Tacticas

- **Cache por banda, no por individuo.** El hash de `(center_band, width_band)` es el mismo para entidades con nichos similares — región entera de la población tiene hit rate alto.
- **Snapshot anti-aliasing.** El sistema toma un snapshot antes de modificar componentes para evitar el problema `Query<&mut>` de Bevy con lecturas simultáneas.
- **4D Hutchinson como arrays fijos.** `[f32; 4]` — sin Vec, sin heap. Dimensiones: freq, x, z, phase temporal.
- **Exclusión competitiva emerge sola.** No hay código que "decida" quién excluye a quién. El displacement acumulado a lo largo de ticks causa divergencia natural.

---

## NO hace

- No modifica genes (`InferenceProfile`) — el desplazamiento es fenotípico (ET-6 epigenetics lo haría hereditario).
- No implementa hibridación entre nichos — eso es reproducción sexual (T4+).
- No visualiza el hipervolumen — eso es GS-7 VisualContract.

---

## Dependencias

- ET-6 `EpigeneticState` — la expresión epigenética puede modificar `NicheProfile.center` (plasticidad de nicho).
- `world/SpatialIndex` — detectar competidores en radio.
- `layers/engine.rs::AlchemicalEngine` — `base_intake()` como proxy de demanda de recursos.

---

## Criterios de Aceptación

- `niche_overlap([0,0,0,0], [1,1,1,1], [0,0,0,0], [1,1,1,1])` → `1.0` (nichos idénticos).
- `niche_overlap([0,0,0,0], [1,1,1,1], [3,3,3,3], [1,1,1,1])` → `0.0` (sin contacto).
- `niche_breadth([2.0, 2.0, 2.0, 2.0])` → `2.0`.
- `character_displacement(0.0, 1.0, 0.01)` → `-0.01` (alejarse del competidor).
- `character_displacement(1.0, 0.0, 0.01)` → `+0.01`.
- Test: dos entidades con nicho idéntico → desplazamiento divergente en N ticks.
- Test: entidades en nichos distintos (overlap=0) → sin modificación.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-6 Epigenetic Expression — plasticidad fenotípica del nicho
- `src/worldgen/field_grid.rs` — `EnergyFieldGrid` como recurso espacial
- Blueprint §T2-5: "Multidimensional Niche", Hutchinson hypervolume
