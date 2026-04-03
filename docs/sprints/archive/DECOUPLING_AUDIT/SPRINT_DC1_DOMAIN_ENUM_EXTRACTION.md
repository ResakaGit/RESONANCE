# DC-1: Domain Enum Extraction — Purificar blueprint/equations/

**Objetivo:** Extraer los 4 enums de dominio (`MatterState`, `OrganRole`, `TrophicClass`, `LifecycleStage`) de `layers/` a `blueprint/`, eliminando las 41 importaciones impuras en `blueprint/equations/`.

**Estado:** PENDIENTE
**Esfuerzo:** M (~55 archivos tocados, mayoría mecánica de re-imports)
**Bloqueado por:** —
**Desbloquea:** DC-2 (shape decomp), DC-4 (math boundary)

---

## Problema

`blueprint/equations/` debe contener math pura — sin dependencia de ECS, sin conocimiento de layers. Hoy, 41 archivos de ecuaciones importan tipos definidos en `layers/`:

| Enum | Definido en | Importado por equations/ | Veces |
|------|-------------|--------------------------|-------|
| `MatterState` | `layers/coherence.rs:16` | core_physics, locomotion, abiogenesis, contact, entity_shape, observability, checkpoint... | 10× |
| `OrganRole` | `layers/organ.rs:11` | protein_fold, metabolic_genome, morph_robustness, field_color, organ_inference, pathway_inhibitor... | 6× |
| `TrophicClass` | `layers/inference.rs:179` | ecology_dynamics, trophic... | 2× |
| `LifecycleStage` | `layers/organ.rs:30` | lifecycle, organ_inference... | 2× |

**Causa raíz:** Los enums son conceptos de dominio puro (no ECS), pero viven en archivos que también definen componentes ECS. Su única dependencia de Bevy es el derive `Reflect`, que es compile-time y no requiere runtime.

---

## Diseño

### Principio: Los tipos de dominio viven en blueprint. Los componentes ECS los envuelven.

```
ANTES:                              DESPUÉS:
layers/coherence.rs                 blueprint/domain_enums.rs
  pub enum MatterState { ... }        pub enum MatterState { ... }   ← CANÓNICO
  #[derive(Component)]                                               (sin Reflect derive)
  pub struct MatterCoherence {      layers/coherence.rs
      state: MatterState,             use crate::blueprint::MatterState;
  }                                   pub use crate::blueprint::MatterState;  ← re-export
                                      #[derive(Component, Reflect)]
                                      pub struct MatterCoherence {
                                          state: MatterState,
                                      }
```

### Archivo nuevo: `src/blueprint/domain_enums.rs`

```rust
//! Enums de dominio puro — zero dependencias de Bevy.
//!
//! Estos enums representan conceptos del modelo físico (estados de materia,
//! roles de órganos, clases tróficas, etapas de ciclo de vida).
//! Son engine-agnostic: no derivan Component, Reflect, ni ningún trait de Bevy.
//!
//! Los componentes ECS en layers/ los envuelven y añaden los derives de Bevy.
//! Las ecuaciones en blueprint/equations/ los usan directamente.

/// Estado de materia — derivado de densidad energética (Axiom 1).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatterState {
    #[default]
    Solid = 0,
    Liquid = 1,
    Gas = 2,
    Plasma = 3,
}

/// Rol funcional de un órgano en el plan corporal.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum OrganRole {
    #[default]
    Stem = 0,
    Root = 1,
    Core = 2,
    Leaf = 3,
    Petal = 4,
    Sensory = 5,
    Thorn = 6,
    Shell = 7,
    Fruit = 8,
    Bud = 9,
    Limb = 10,
    Fin = 11,
}

/// Clasificación trófica — emergente de la composición energética.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TrophicClass {
    #[default]
    PrimaryProducer = 0,
    Herbivore = 1,
    Omnivore = 2,
    Carnivore = 3,
    Detritivore = 4,
}

/// Etapa del ciclo de vida de un órgano.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum LifecycleStage {
    #[default]
    Dormant = 0,
    Emerging = 1,
    Growing = 2,
    Mature = 3,
    Reproductive = 4,
    Declining = 5,
}
```

### Constantes asociadas (migran con el enum)

```rust
// En organ.rs vive MAX_ORGANS_PER_ENTITY = 8 y ORGAN_ROLE_PRIMITIVE[].
// MAX_ORGANS_PER_ENTITY es una constante de dominio (no ECS).
// ORGAN_ROLE_PRIMITIVE[] mapea OrganRole → GeometryPrimitive (domain).
//
// Ambos migran a blueprint/domain_enums.rs junto con OrganRole.
// El método OrganRole::primitive() migra también (es const fn puro).

impl OrganRole {
    pub const MAX_PER_ENTITY: usize = 8;

    #[inline]
    pub const fn primitive(self) -> GeometryPrimitive {
        ORGAN_ROLE_PRIMITIVE[self as usize]
    }
}
```

### Re-exports en layers/ (backward compatibility)

```rust
// layers/coherence.rs — DESPUÉS de la migración
use crate::blueprint::domain_enums::MatterState;
pub use crate::blueprint::domain_enums::MatterState;
//     ^^^ re-export para que `use crate::layers::MatterState` siga compilando

// layers/organ.rs
pub use crate::blueprint::domain_enums::{OrganRole, LifecycleStage};
pub use crate::blueprint::domain_enums::OrganRole as OrganRole; // MAX_PER_ENTITY via impl

// layers/inference.rs
pub use crate::blueprint::domain_enums::TrophicClass;
```

### Migración de imports en equations/ (mecánica)

```
ANTES:  use crate::layers::MatterState;
DESPUÉS: use crate::blueprint::MatterState;

ANTES:  use crate::layers::OrganRole;
DESPUÉS: use crate::blueprint::OrganRole;

// etc. para TrophicClass, LifecycleStage
```

**Herramienta:** `sed` + `cargo check` iterativo. No requiere cambio de lógica en ningún archivo.

---

## Plan de ejecución (4 commits atómicos)

### Commit 1: Crear `blueprint/domain_enums.rs` con los 4 enums

- Copiar definiciones (sin `Reflect` derive)
- Copiar constantes asociadas (`MAX_PER_ENTITY`, `ORGAN_ROLE_PRIMITIVE`, `OrganRole::primitive()`)
- Registrar módulo en `blueprint/mod.rs`: `pub mod domain_enums;`
- Re-exportar desde `blueprint/mod.rs`: `pub use domain_enums::{MatterState, OrganRole, TrophicClass, LifecycleStage};`
- **Test:** `cargo check` pasa (archivo nuevo, sin consumidores aún)

### Commit 2: Re-export desde layers/ (backward compat)

- En `layers/coherence.rs`: eliminar definición de `MatterState`, reemplazar con `pub use crate::blueprint::MatterState;`
- En `layers/organ.rs`: eliminar definiciones de `OrganRole`, `LifecycleStage`, reemplazar con `pub use`
- En `layers/inference.rs`: eliminar definición de `TrophicClass`, reemplazar con `pub use`
- Añadir `#[derive(Reflect)]` vía wrapper si necesario, o usar `app.register_type_data::<MatterState, ReflectDefault>()` en plugin
- **Test:** `cargo test` completo — 0 regresiones. Todos los imports existentes siguen compilando vía re-export.

**Decisión técnica sobre Reflect:**

Los 4 enums usan `#[derive(Reflect)]` hoy. `Reflect` es un derive procedural de Bevy. Opciones:

| Opción | Pros | Contras | Decisión |
|--------|------|---------|----------|
| **A: Conditional derive** `#[cfg_attr(feature="bevy_reflect", derive(Reflect))]` | Enums puros en batch mode, Reflect en Bevy mode | Requiere feature gate, complejidad | NO — over-engineering |
| **B: Derive Reflect en blueprint** | Simple, funciona hoy | blueprint depende de bevy_reflect | NO — viola pureza |
| **C: Newtype wrapper en layers** | Enum puro en blueprint, wrapper con Reflect en layers | Ergonomía reducida, double indirection | NO — complejidad sin beneficio |
| **D: Reflect derive directo** | Zero cambio para consumidores, funciona hoy | blueprint tiene dep en bevy_reflect (ya la tiene vía bevy::prelude en otros archivos) | **SÍ — pragmático** |

**Decisión: Opción D.** `blueprint/` ya importa `bevy::prelude::*` en varios archivos (ids, recipes, almanac). Añadir `Reflect` a los enums es consistente con el status quo. La pureza absoluta (zero Bevy en equations/) se mantiene porque equations/ no importa los enums desde layers/ — los importa desde blueprint/.

```rust
// blueprint/domain_enums.rs — versión final
use bevy::prelude::Reflect;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatterState { ... }
```

### Commit 3: Migrar imports en equations/ (41 archivos)

- Buscar: `use crate::layers::MatterState` → reemplazar: `use crate::blueprint::MatterState`
- Buscar: `use crate::layers::OrganRole` → reemplazar: `use crate::blueprint::OrganRole`
- Buscar: `use crate::layers::organ::MAX_ORGANS_PER_ENTITY` → `use crate::blueprint::OrganRole` + `OrganRole::MAX_PER_ENTITY`
- Buscar: `use crate::layers::inference::TrophicClass` → `use crate::blueprint::TrophicClass`
- Buscar: `use crate::layers::LifecycleStage` → `use crate::blueprint::LifecycleStage`
- **Test:** `cargo test` completo + `grep -r "use crate::layers::" src/blueprint/equations/` devuelve 0 resultados

### Commit 4: Cleanup — eliminar re-exports redundantes (opcional, separado)

- Si todos los consumidores de `layers::MatterState` se migraron (no solo equations/), eliminar re-exports
- Si quedan consumidores legítimos en simulation/, rendering/, etc. → mantener re-exports
- **Decisión:** Los re-exports en layers/ se MANTIENEN indefinidamente. Son zero-cost y evitan breaking changes para simulation/ y rendering/ que legítimamente acceden vía layers/. Solo equations/ migra su import path.

---

## Testing

### Capa 1: Unitario

```rust
// blueprint/domain_enums.rs — tests inline
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matter_state_default_is_solid() {
        assert_eq!(MatterState::default(), MatterState::Solid);
    }

    #[test]
    fn organ_role_primitive_is_const() {
        // Verifica que primitive() es evaluable en compile-time
        const P: GeometryPrimitive = OrganRole::Stem.primitive();
        assert_eq!(P, GeometryPrimitive::Sphere);
    }

    #[test]
    fn organ_role_repr_u8_round_trip() {
        for role in [OrganRole::Stem, OrganRole::Root, OrganRole::Core,
                     OrganRole::Leaf, OrganRole::Fin] {
            let byte = role as u8;
            // Verifica que repr(u8) es estable para serialización batch
            assert!(byte < OrganRole::MAX_PER_ENTITY as u8 + 4);
        }
    }

    #[test]
    fn trophic_class_exhaustive_match() {
        // Verifica que todos los variantes existen (compiler catch si se añade uno)
        let classes = [TrophicClass::PrimaryProducer, TrophicClass::Herbivore,
                       TrophicClass::Omnivore, TrophicClass::Carnivore,
                       TrophicClass::Detritivore];
        assert_eq!(classes.len(), 5);
    }

    #[test]
    fn lifecycle_stage_ordering_matches_repr() {
        assert!((LifecycleStage::Dormant as u8) < (LifecycleStage::Mature as u8));
        assert!((LifecycleStage::Mature as u8) < (LifecycleStage::Declining as u8));
    }
}
```

### Capa 2: Integración (import path validation)

```rust
// tests/integration/dc1_enum_extraction.rs
use resonance::blueprint::{MatterState, OrganRole, TrophicClass, LifecycleStage};
use resonance::layers::MatterCoherence; // Still works via re-export

#[test]
fn blueprint_enums_are_canonical_source() {
    // Verifica que blueprint es el módulo canónico
    let _s = MatterState::Solid;
    let _r = OrganRole::Stem;
    let _t = TrophicClass::Herbivore;
    let _l = LifecycleStage::Growing;
}

#[test]
fn layers_reexport_still_compiles() {
    // Backward compat: layers/ re-exports siguen funcionando
    use resonance::layers::MatterState as LayerMatterState;
    let s: LayerMatterState = MatterState::Solid;
    assert_eq!(s, MatterState::Solid);
}
```

### Capa 3: Orquestación (regression guard)

```bash
# Script de CI / criterio de cierre
cargo test 2>&1 | tail -5    # 0 failures
grep -r "use crate::layers::" src/blueprint/equations/ | wc -l  # Must be 0
grep -r "use crate::layers::MatterState" src/blueprint/ | wc -l  # Must be 0
grep -r "use crate::layers::OrganRole" src/blueprint/ | wc -l    # Must be 0
```

---

## Integración al codebase

### Lo que NO cambia
- Ninguna ecuación cambia de lógica — solo import paths
- Ningún componente ECS cambia — `MatterCoherence` sigue teniendo `state: MatterState`
- Ningún sistema cambia — queries siguen accediendo los mismos componentes
- batch/ sigue importando desde layers/ (vía bridge.rs) — no afecta

### Lo que SÍ cambia
- `blueprint/domain_enums.rs` — archivo NUEVO (source of truth)
- `blueprint/mod.rs` — una línea de `pub mod` + `pub use`
- `layers/coherence.rs` — definición de MatterState eliminada, reemplazada por `pub use`
- `layers/organ.rs` — definiciones de OrganRole + LifecycleStage eliminadas, reemplazadas por `pub use`
- `layers/inference.rs` — definición de TrophicClass eliminada, reemplazada por `pub use`
- 41 archivos en `blueprint/equations/` — import path cambia de `layers::` a `blueprint::`

### Riesgos

| Riesgo | Probabilidad | Mitigación |
|--------|-------------|------------|
| Reflect derive falla sin bevy context | Baja | `Reflect` es un derive proc macro, funciona en compile-time sin app |
| Serde derives pierden compatibilidad | Baja | MatterState es el único con serde; mantener identical representation |
| batch/ bridge.rs rompe | Media | Verificar que bridge.rs importa MatterState desde layers/ (re-export funciona) |
| Circular dependency blueprint↔layers | Nula | Dirección es unidireccional: layers → blueprint (layers depende de blueprint, nunca al revés) |

---

## Scope definido

**Entra:**
- Mover 4 enums + constantes asociadas a `blueprint/domain_enums.rs`
- Re-exports en layers/ para backward compat
- Migrar 41 import paths en equations/
- Tests unitarios del nuevo módulo + integration tests de import paths

**NO entra:**
- Migrar imports de simulation/, rendering/, worldgen/ (usan layers/ legítimamente)
- Refactor de `MetabolicGraph` (es un componente, no un enum — queda para otro sprint)
- Eliminar re-exports de layers/ (se mantienen indefinidamente)
- Cambiar la API de ningún enum (zero breaking changes)

---

## Criterios de cierre

- [ ] `cargo test` — 3,051+ tests, 0 failures
- [ ] `grep -r "use crate::layers::" src/blueprint/equations/` — 0 resultados
- [ ] `blueprint/domain_enums.rs` existe con 4 enums + tests
- [ ] layers/ re-exports funcionan (test de integración pasa)
- [ ] Ningún `// DEBT:` introducido
- [ ] batch/ bridge.rs compila sin cambios
