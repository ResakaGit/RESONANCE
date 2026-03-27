# Sprint ET-5 — Obligate Symbiosis: Dependencia Energética Mutua

**Módulo:** `src/layers/symbiosis.rs` (nuevo), `src/blueprint/equations/emergence/symbiosis.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T2-1. **Onda:** 0.
**BridgeKind:** `SymbiosisBridge` — cache Small(128), clave `(entity_a_band, entity_b_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

Dos entidades cuyos campos de extracción se vuelven mutuamente dependientes forman un sistema energético acoplado con propiedades de estabilidad emergentes. La dependencia es física: `F_intake(A sin B) < F_dissipation(A)` → A no puede sobrevivir sola.

```
mutualism ↔ F_intake(A|B) > F_intake(A) AND F_intake(B|A) > F_intake(B)
parasitism ↔ F_intake(parasite) ∝ host_damage  AND  F_intake(host) ↓
obligate   ↔ F_intake(A sin B) < F_dissipation(A)
```

---

## Responsabilidades

### ET-5A: Ecuaciones

```rust
// src/blueprint/equations/emergence/symbiosis.rs

/// Beneficio de la simbiosis: qe/tick adicional cuando el partner está presente.
pub fn mutualism_benefit(own_intake: f32, partner_bonus_factor: f32) -> f32 {
    own_intake * partner_bonus_factor
}

/// Costo del parasitismo: qe/tick extraído del host por el parásito.
pub fn parasitism_drain(host_qe: f32, drain_rate: f32) -> f32 {
    host_qe * drain_rate
}

/// ¿La dependencia es obligada? Sí si el intake sin partner cae bajo la disipación base.
pub fn is_obligate_dependency(
    intake_without_partner: f32,
    base_dissipation: f32,
) -> bool {
    intake_without_partner < base_dissipation
}

/// Estabilidad de la relación simbiótica (Nash).
/// Retorna true si ninguna parte gana más rompiendo la relación.
pub fn is_symbiosis_stable(
    a_with_b: f32, a_without_b: f32,
    b_with_a: f32, b_without_a: f32,
) -> bool {
    a_with_b >= a_without_b && b_with_a >= b_without_a
}

/// Coevolution pressure: cuánto presiona B a A a adaptarse.
pub fn coevolution_pressure(
    extraction_b_on_a: f32,
    resistance_a: f32,
) -> f32 {
    (extraction_b_on_a - resistance_a).max(0.0)
}
```

### ET-5B: Componente

```rust
// src/layers/symbiosis.rs

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]  // relaciones son pocas y transientes
pub struct SymbiosisLink {
    pub partner_id:     u32,     // WorldEntityId del partner
    pub relationship:   SymbiosisType,
    pub bonus_factor:   f32,     // amplificador de intake mutualista
    pub drain_rate:     f32,     // tasa de extracción parasítica
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq)]
pub enum SymbiosisType {
    Mutualism,   // ambos se benefician
    Parasitism,  // parásito extrae del host
    Commensalism,// uno se beneficia, el otro neutro
}
```

### ET-5C: Sistema

```rust
/// Aplica efectos de simbiosis: bonus de intake o drain parasítico.
/// Phase::ChemicalLayer — after catalysis, before metabolic.
pub fn symbiosis_effect_system(
    mut entities: Query<(&WorldEntityId, &mut BaseEnergy, &mut AlchemicalEngine, &SymbiosisLink)>,
    partners: Query<(&WorldEntityId, &BaseEnergy)>,
) {
    for (self_id, mut energy, mut engine, link) in &mut entities {
        let partner_present = partners.iter()
            .any(|(id, _)| id.0 == link.partner_id && id.0 != self_id.0);
        match link.relationship {
            SymbiosisType::Mutualism => {
                if partner_present {
                    let bonus = symbiosis_eq::mutualism_benefit(engine.base_intake(), link.bonus_factor);
                    let new_intake = engine.intake() + bonus;
                    if engine.intake() != new_intake { engine.set_intake(new_intake); }
                }
            }
            SymbiosisType::Parasitism => {
                // El parásito drena al host — el sistema del parásito llama esto sobre el host
                let drain = symbiosis_eq::parasitism_drain(energy.qe(), link.drain_rate);
                let new_qe = (energy.qe() - drain).max(0.0);
                if energy.qe() != new_qe { energy.set_qe(new_qe); }
            }
            SymbiosisType::Commensalism => { /* neutro para el host */ }
        }
    }
}
```

### ET-5D: Constantes y BridgeKind

```rust
pub struct SymbiosisBridge;
impl BridgeKind for SymbiosisBridge {}

pub const SYMBIOSIS_DEFAULT_MUTUALISM_BONUS: f32 = 0.2;  // +20% intake
pub const SYMBIOSIS_DEFAULT_PARASITISM_DRAIN: f32 = 0.05; // 5% qe/tick
```

---

## Tacticas

- **SparseSet:** las relaciones simbióticas son pocas. SparseSet → iteración rápida sobre el subset.
- **BridgeCache para `mutualism_benefit`.** El bonus es función del intake base (cambia lentamente). Cache key por banda de intake.
- **Coevolución emerge del juego existente.** `coevolution_pressure` no es un sistema — es una ecuación que el sistema de homeostasis puede usar para ajustar frecuencias (L12 existente).

---

## Criterios de Aceptación

- `is_symbiosis_stable(10, 5, 10, 5)` → `true`. `is_symbiosis_stable(5, 10, 10, 5)` → `false`.
- `is_obligate_dependency(2.0, 5.0)` → `true`. `is_obligate_dependency(10.0, 5.0)` → `false`.
- Test: entidad con Mutualism y partner presente → intake aumenta.
- Test: entidad con Mutualism y partner ausente → intake sin cambio.
- Test: parásito drena qe del host en cada tick.
- `cargo test --lib` sin regresión.

---

## Referencias

- `src/layers/structural_link.rs` — patrón SparseSet para vínculos entre entidades
- Blueprint §T2-1: "Obligate Symbiosis and Parasitism"
