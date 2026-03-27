# Sprint G5 — Pathfinding (NavMesh + Flowfield)

**Tipo:** Feature — navegacion inteligente para entidades.
**Riesgo:** ALTO — introduce dependencia de crate externo, modifica movimiento de heroes.
**Onda:** A — Independiente. Habilita movimiento real con obstaculos.
**Estado:** Pendiente

## Principio Filosofico

> **Pathfinding alimenta WillActuator (L7), no Transform.** La ruta se convierte en `movement_intent`. El movimiento fisico sigue siendo `will_force → FlowVector → Transform` via ecuaciones puras. Un heroe en zona viscosa (L6) se mueve mas lento por la misma ruta — sin codigo extra.

## Objetivo

Reemplazar movimiento en linea recta por pathfinding real. Heroes usan NavMesh (precision individual). Creeps/minions usan flowfields (eficiencia masiva). Avoidance local previene overlap. La ruta solo define DIRECCION; la velocidad emerge de la fisica energetica (L3, L1, L6).

## Estado actual en Resonance

- Click-to-move basico existe en `src/runtime_platform/click_to_move/`
- `WillActuator.movement_intent` define direccion de movimiento
- `movement_system` integra velocity en `Phase::Physics`
- **No hay:** pathfinding, obstacle avoidance, NavMesh, flowfields

## Responsabilidades

### Paso 1 — Integrar `oxidized_navigation`

Agregar dependencia en `Cargo.toml`:
```toml
oxidized_navigation = "0.12"  # verificar version compatible con Bevy 0.15
```

### Paso 2 — Generar NavMesh desde terreno

Crear `src/simulation/pathfinding.rs`:

1. En startup (despues de worldgen), generar NavMesh desde la geometria del terreno
2. Marcar obstaculos como non-walkable
3. `OxidizedNavigationPlugin` registra el NavMesh como Resource

```rust
// Componente para entidades que usan pathfinding
#[derive(Component)]
pub struct NavAgent {
    pub speed: f32,
    pub radius: f32,
}

// Resource: path calculado por agente
#[derive(Component)]
pub struct NavPath {
    pub waypoints: Vec<Vec3>,
    pub current_index: usize,
}
```

### Paso 3 — Sistema de path request

Cuando el jugador hace click-to-move:
1. `click_to_move` detecta posicion target en ground
2. En vez de setear `movement_intent` directamente, emitir `PathRequestEvent`
3. `pathfinding_system` calcula ruta via NavMesh
4. Resultado se almacena en `NavPath` component

### Paso 4 — Sistema de path following

```rust
/// Sigue waypoints del NavPath. Corre en Phase::Input.
pub fn path_follow_system(
    mut query: Query<(&mut WillActuator, &Transform, &NavPath, &NavAgent)>,
) {
    for (mut will, transform, path, agent) in &mut query {
        if let Some(next_wp) = path.waypoints.get(path.current_index) {
            let dir = (*next_wp - transform.translation).normalize_or_zero();
            will.movement_intent = Vec2::new(dir.x, dir.z);
            // Avanzar waypoint si estamos cerca
        } else {
            will.movement_intent = Vec2::ZERO; // llegamos
        }
    }
}
```

**Clave:** El pathfinding NO reemplaza `movement_system`. Solo alimenta `WillActuator`. El pipeline de fisica sigue igual.

### Paso 5 — Flowfields para creeps (futuro)

Opcional en este sprint. Si se implementa:
- `bevy_flowfield_tiles_plugin` para creeps/minions
- Un flowfield por carril (lane), recalculado cuando un obstaculo cambia
- Mas eficiente que NavMesh individual para 50+ entidades

### Paso 6 — Local avoidance

Basico: separacion simple entre agentes cercanos (boids separation). Avanzado: RVO2 (Reciprocal Velocity Obstacles).

Para MVP: solo separacion simple:
```rust
fn avoidance_system(
    mut query: Query<(&mut WillActuator, &Transform, &NavAgent)>,
) {
    // Para cada par cercano, agregar fuerza de separacion a movement_intent
}
```

## Tacticas

- **NavMesh first.** Flowfields y avoidance son polish. El MVP es: click → path → follow.
- **No romper movement_system.** Pathfinding alimenta `WillActuator`, no reemplaza la integracion de fisica.
- **Verificar compatibilidad Bevy 0.15.** `oxidized_navigation` debe soportar Bevy 0.15. Si no, evaluar `vleue_navigator` como alternativa.
- **Recalculo lazy.** No recalcular path cada frame. Solo cuando: nuevo click, obstaculo aparece/desaparece, o path queda bloqueado.

## NO hace

- No modifica `movement_system` ni `FlowVector`.
- No implementa creep AI (eso es otro sprint).
- No implementa terrain avoidance vertical (solo XZ).
- No modifica el sistema de colisiones.
- No implementa A* manual — usa crate existente.

## Criterio de aceptacion

- [ ] `oxidized_navigation` (o alternativa) integrado en `Cargo.toml`
- [ ] NavMesh se genera desde terreno en startup
- [ ] Click-to-move usa pathfinding en vez de linea recta
- [ ] Hero navega alrededor de obstaculos
- [ ] Path se recalcula al clickear nueva posicion
- [ ] `NavAgent` y `NavPath` componentes existen
- [ ] `cargo check` pasa
- [ ] `cargo test` — tests nuevos para path following
- [ ] Demo arena: hero camina alrededor de obstaculos al clickear

## Esfuerzo estimado

~4-6 horas. La integracion con `oxidized_navigation` es el paso critico — depende de la compatibilidad con Bevy 0.15 y la geometria del terreno.
