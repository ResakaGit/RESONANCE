# Sprint GS-4 — Pack Dynamics: Formación de Manada y Cohesión

**Modulo:** `src/simulation/metabolic/social_communication.rs` (extensión), `src/blueprint/equations/pack_dynamics.rs` (nuevo), `src/blueprint/constants/pack_dynamics.rs` (nuevo)
**Tipo:** Ecuaciones puras + extensión de sistemas sociales.
**Onda:** A — Requiere GS-3 (Nash targeting) + `PackMembership` existente.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `layers/social_communication.rs` — `PackMembership { pack_id: u32, role: PackRole }`, `PackRole { Leader | Follower | Scout }`. Componente de pertenencia ya existe.
- `simulation/metabolic/social_communication.rs` — `pack_formation_system` ya corre en `Phase::MetabolicLayer`. Gestiona cohesión básica.
- `simulation/behavior.rs` — `BehaviorMode::FocusFire { target, team_priority }` (GS-3), `BehaviorMode::Regroup { rally_pos }` (stub pendiente).
- `blueprint/equations/tactical_ai.rs` — `threat_magnitude`, `threat_gradient` (GS-3). Listos para consumir.
- `world/SpatialIndex` — `query_radius` para vecinos de pack.
- `layers/flow.rs::FlowVector` — velocidad de la entidad. Cohesión modifica esto.

**Lo que NO existe:**

1. **Formación de manada.** No hay función que calcule el punto de rally basado en la distribución del pack.
2. **Fuerza de cohesión.** No hay vector de fuerza que empuje hacia el centro del pack.
3. **Respuesta de amenaza grupal.** `threat_gradient` existe pero nadie lo usa para coordinar la huída.
4. **Activación de `Regroup`.** El stub de `BehaviorMode::Regroup` no tiene sistema que lo active.
5. **Dominancia de liderazgo.** El `Leader` no influye más que los `Follower` en la dirección del pack.

---

## Objetivo

Extender el sistema social para que el pack actúe como unidad táctica: cohesión hacia el líder, respuesta coordinada a amenazas (flee colectivo), y activación de `Regroup` cuando el pack está disperso. La formación emerge de fuerzas físicas, no de reglas.

```
pack_center   = weighted_avg(positions, weight=leadership_weight)
cohesion_vec  = normalize(pack_center - self_pos) × cohesion_strength
flee_vec      = -threat_gradient(self_pos, enemies) × flee_strength
intent_vector = cohesion_vec + flee_vec  (→ WillActuator)
```

---

## Responsabilidades

### GS-4A: Ecuaciones de pack dynamics

```rust
// src/blueprint/equations/pack_dynamics.rs

/// Centro de masa del pack, ponderado por el rol (líder pesa más).
/// positions: Vec de (pos, weight). Retorna [0,0] si vacío.
pub fn pack_center(positions: &[([f32; 2], f32)]) -> [f32; 2] {
    let total_weight: f32 = positions.iter().map(|(_, w)| w).sum();
    if total_weight <= 0.0 { return [0.0, 0.0]; }
    let cx = positions.iter().map(|(p, w)| p[0] * w).sum::<f32>() / total_weight;
    let cy = positions.iter().map(|(p, w)| p[1] * w).sum::<f32>() / total_weight;
    [cx, cy]
}

/// Fuerza de cohesión: vector hacia el centro ponderado por distancia.
/// Fuerte cuando lejos, débil cuando cerca (zona muerta = dead_zone_radius).
pub fn cohesion_force(
    self_pos: [f32; 2],
    center: [f32; 2],
    dead_zone_radius: f32,
    max_force: f32,
) -> [f32; 2] {
    let dx = center[0] - self_pos[0];
    let dy = center[1] - self_pos[1];
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= dead_zone_radius { return [0.0, 0.0]; }
    let factor = ((dist - dead_zone_radius) / dist).min(1.0) * max_force;
    [dx / dist * factor, dy / dist * factor]
}

/// ¿Está el pack disperso? Sí si la desviación estándar de distancias supera el umbral.
pub fn is_pack_dispersed(
    positions: &[[f32; 2]],
    center: [f32; 2],
    dispersion_threshold: f32,
) -> bool {
    if positions.len() < 2 { return false; }
    let mean_dist = positions.iter()
        .map(|p| {
            let dx = p[0] - center[0];
            let dy = p[1] - center[1];
            (dx * dx + dy * dy).sqrt()
        })
        .sum::<f32>() / positions.len() as f32;
    mean_dist > dispersion_threshold
}

/// Vector de intent resultante: cohesión + huída anti-amenaza.
/// Retorna vector normalizado. Prioridad: amenaza > cohesión.
pub fn pack_intent_vector(
    cohesion: [f32; 2],
    threat_grad: [f32; 2],
    threat_magnitude: f32,
    cohesion_weight: f32,
    flee_weight: f32,
) -> [f32; 2] {
    // Amenaza: alejarse de la fuente (negar gradiente)
    let flee_x = -threat_grad[0] * threat_magnitude * flee_weight;
    let flee_y = -threat_grad[1] * threat_magnitude * flee_weight;
    let intent_x = cohesion[0] * cohesion_weight + flee_x;
    let intent_y = cohesion[1] * cohesion_weight + flee_y;
    let mag = (intent_x * intent_x + intent_y * intent_y).sqrt();
    if mag < 0.001 { [0.0, 0.0] } else { [intent_x / mag, intent_y / mag] }
}
```

### GS-4B: Extensión de BehaviorMode::Regroup

```rust
// src/simulation/behavior.rs — completar el stub de GS-3:

/// Activa Regroup cuando el pack está disperso y sin amenaza inmediata.
/// Phase::Input, in_set(BehaviorSet::Decide), after nash_target_select_system.
pub fn pack_regroup_system(
    mut agents: Query<(
        &PackMembership,
        &Transform,
        &SensoryAwareness,
        &mut BehaviorMode,
    ), With<BehavioralAgent>>,
    pack_members: Query<(&PackMembership, &Transform)>,
    config: Res<PackDynamicsConfig>,
) {
    // Collect pack center per pack_id
    // For each agent: if dispersed and threat < threshold → Regroup { rally_pos }
    for (membership, transform, awareness, mut mode) in &mut agents {
        let in_regroup_eligible = matches!(
            *mode,
            BehaviorMode::Idle | BehaviorMode::Forage | BehaviorMode::Hunt { .. }
        );
        if !in_regroup_eligible { continue; }

        let positions: Vec<[f32; 2]> = pack_members.iter()
            .filter(|(m, _)| m.pack_id == membership.pack_id)
            .map(|(_, t)| [t.translation.x, t.translation.z])
            .collect();

        let center = pack_dynamics_eq::pack_center(
            &positions.iter().map(|p| (*p, 1.0f32)).collect::<Vec<_>>()
        );

        if pack_dynamics_eq::is_pack_dispersed(&positions, center, config.dispersion_threshold) {
            let rally = Vec2::new(center[0], center[1]);
            let new_mode = BehaviorMode::Regroup { rally_pos: rally };
            if *mode != new_mode {
                *mode = new_mode;
            }
        }
    }
}
```

### GS-4C: Sistema de cohesión de pack

```rust
/// Aplica fuerza de cohesión + respuesta a amenaza a entidades del pack.
/// Phase::MetabolicLayer, after pack_regroup_system, before pack_formation_apply.
pub fn pack_cohesion_force_system(
    mut agents: Query<(
        &PackMembership,
        &Transform,
        &SensoryAwareness,
        &Faction,
        &mut WillActuator,
    ), With<BehavioralAgent>>,
    pack_members: Query<(&PackMembership, &Transform, Option<&PackRole>)>,
    all_entities: Query<(&Transform, &InferenceProfile, &Faction)>,
    config: Res<PackDynamicsConfig>,
) {
    for (membership, transform, awareness, faction, mut will) in &mut agents {
        // Calcular centro ponderado del pack (líderes peso 2.0, followers 1.0)
        let weighted_positions: Vec<([f32; 2], f32)> = pack_members.iter()
            .filter(|(m, _, _)| m.pack_id == membership.pack_id)
            .map(|(_, t, role)| {
                let w = if matches!(role, Some(PackRole::Leader)) { 2.0 } else { 1.0 };
                ([t.translation.x, t.translation.z], w)
            })
            .collect();
        let center = pack_dynamics_eq::pack_center(&weighted_positions);

        // Fuerza de cohesión
        let self_pos = [transform.translation.x, transform.translation.z];
        let cohesion = pack_dynamics_eq::cohesion_force(
            self_pos, center,
            config.cohesion_dead_zone,
            config.max_cohesion_force,
        );

        // Gradiente de amenaza desde GS-3
        let enemy_positions: Vec<([f32; 2], f32)> = awareness.detected_entities.iter()
            .filter_map(|&e| {
                let (t, profile, f) = all_entities.get(e).ok()?;
                if f == faction { return None; }
                Some(([t.translation.x, t.translation.z], profile.extraction_capacity()))
            })
            .collect();
        let threat_grad = tactical_ai_eq::threat_gradient(self_pos, &enemy_positions);
        let threat_mag = tactical_ai_eq::threat_magnitude(
            self_pos, &enemy_positions, config.threat_radius,
        );

        let intent = pack_dynamics_eq::pack_intent_vector(
            cohesion, threat_grad, threat_mag,
            config.cohesion_weight, config.flee_weight,
        );

        will.set_social_intent(Vec2::new(intent[0], intent[1]));
    }
}
```

### GS-4D: Config y constantes

```rust
// src/blueprint/constants/pack_dynamics.rs

pub const PACK_DISPERSION_THRESHOLD: f32 = 15.0;   // unidades de distancia
pub const PACK_COHESION_DEAD_ZONE: f32 = 3.0;       // zona muerta alrededor del centro
pub const PACK_MAX_COHESION_FORCE: f32 = 5.0;       // magnitud máxima
pub const PACK_FLEE_WEIGHT: f32 = 2.0;              // amenaza > cohesión por default
pub const PACK_COHESION_WEIGHT: f32 = 1.0;
pub const PACK_THREAT_RADIUS: f32 = 20.0;           // radio para detección de amenazas

// src/simulation/metabolic/social_communication.rs — nuevo Resource
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct PackDynamicsConfig {
    pub dispersion_threshold: f32,
    pub cohesion_dead_zone: f32,
    pub max_cohesion_force: f32,
    pub flee_weight: f32,
    pub cohesion_weight: f32,
    pub threat_radius: f32,
}
```

---

## Tacticas

- **Reutiliza GS-3 sin duplicación.** `threat_gradient` y `threat_magnitude` son funciones puras en `tactical_ai.rs` — GS-4 los consume directamente.
- **Vec sin HashMap.** La búsqueda de miembros del pack es `filter` sobre `Query` — lineal, determinista, sin allocations per-query.
- **WillActuator::social_intent.** La fuerza de pack se expresa como intención social en `WillActuator`, no como modificación directa de `FlowVector`. El sistema de locomotion resuelve la prioridad.
- **Regroup no sobreescribe FocusFire.** El check de elegibilidad en `pack_regroup_system` excluye modos de combate activos.

---

## NO hace

- No implementa formaciones posicionales (V, línea, flanco) — demasiado rígido para física emergente.
- No implementa comunicación de habilidades entre agentes — las habilidades son individuales.
- No modifica `PackMembership` ni `PackRole` — eso es worldgen/spawn.
- No define roles automáticamente — el rol viene del archetype (GS-8).

---

## Dependencias

- GS-3 — `tactical_ai_eq::threat_gradient`, `threat_magnitude`, `BehaviorMode::FocusFire/Regroup`.
- `layers/social_communication.rs` — `PackMembership`, `PackRole` (existentes).
- `layers/will.rs::WillActuator` — punto de escritura para intent social.
- `layers/inference.rs::InferenceProfile` — `extraction_capacity()` para threat weighting.
- `simulation/behavior.rs` — `BehavioralAgent`, `BehaviorSet`, `BehaviorMode`.

---

## Criterios de aceptacion

### GS-4A (Ecuaciones)
- `pack_center(&[(p1, 1.0), (p2, 1.0)])` → promedio geométrico.
- `pack_center(&[(p1, 2.0), (p2, 1.0)])` → más cercano a p1.
- `cohesion_force` dentro de `dead_zone` → `[0.0, 0.0]`.
- `cohesion_force` fuera de `dead_zone` → vector apunta a centro.
- `is_pack_dispersed` con 2 puntos distantes → `true`.
- `is_pack_dispersed` con 2 puntos juntos → `false`.
- `pack_intent_vector` con alta amenaza → flee domina sobre cohesión.

### GS-4B/C (Sistemas)
- Test (MinimalPlugins + GS-3): pack disperso, sin amenaza → `BehaviorMode::Regroup` activado.
- Test: pack en combate (`FocusFire`) → `Regroup` no sobreescribe.
- Test: amenaza detectada → `WillActuator::social_intent` apunta contrario al `threat_gradient`.
- Test: pack unido (dentro de dead_zone) → cohesion_force nula, intent dominado por flee o nulo.

### General
- `cargo test --lib` sin regresión.
- Sin HashMap en hot path.

---

## Referencias

- `src/layers/social_communication.rs` — `PackMembership`, `PackRole`
- `src/simulation/metabolic/social_communication.rs` — sistema base existente
- `src/simulation/behavior.rs` — `BehaviorMode`, `BehaviorSet`, D1
- `src/blueprint/equations/tactical_ai.rs` — GS-3 threat equations
- Blueprint §5: "Pack Formation as Emergent Cohesion"
