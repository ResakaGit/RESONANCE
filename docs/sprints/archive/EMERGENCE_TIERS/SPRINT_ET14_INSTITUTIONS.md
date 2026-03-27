# Sprint ET-14 — Institutions: Coordinación Colectiva que Trasciende Individuos

**Módulo:** `src/simulation/emergence/institutions.rs` (nuevo), `src/blueprint/equations/emergence/institutions.rs` (nuevo)
**Tipo:** Ecuaciones puras + Resource de instituciones + sistema.
**Tier:** T4-1. **Onda:** C.
**BridgeKind:** `InstitutionBridge` — cache Small(64), clave `(rule_hash, compliance_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: qué ya existe

- ET-3 `CulturalMemory` / `MemeEntry` — comportamientos compartidos. Las instituciones son memes con mecanismo de enforcement.
- ET-8 `CoalitionRegistry` — grupos con estabilidad Nash. Las instituciones emergen de coaliciones estables que formalizan sus reglas.
- ET-4 `InfrastructureInvestEvent` — inversión colectiva en infraestructura. Las instituciones coordinan esta inversión.
- `layers/social_communication.rs::PackMembership` — grupos base.

**Lo que NO existe:**
1. Reglas que persisten más allá de las entidades que las crearon.
2. Mecanismo de enforcement que penaliza defectores.
3. `InstitutionRegistry` — catálogo de instituciones activas con miembros.
4. Distribución de recursos entre miembros según reglas institucionales.

---

## Objetivo

Una institución es una regla de coordinación con enforcement: penaliza la defección y subsidia la cooperación. Emerge cuando el ROI de enforcement > costo de administración. Las instituciones trascienden individuos — persisten aunque mueran sus fundadores.

```
institution_stability = compliance_rate × enforcement_efficiency - admin_cost
member_benefit(i) = institution_surplus × allocation_rule(i) - membership_cost(i)
enforcement_pressure(defector) = detection_probability × penalty_rate
```

---

## Responsabilidades

### ET-14A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/institutions.rs

/// Estabilidad de una institución: compliance sostenida menos costos administrativos.
pub fn institution_stability(
    compliance_rate: f32,     // fracción de miembros que cumplen [0,1]
    enforcement_efficiency: f32, // qe recuperado por unidad de enforcement
    admin_cost: f32,          // qe/tick para mantener la institución
) -> f32 {
    compliance_rate * enforcement_efficiency - admin_cost
}

/// Incentivo de cumplimiento: beneficio de cumplir > penalización de defectar.
pub fn compliance_incentive(
    member_benefit: f32,      // qe/tick extra por ser miembro compliance
    defection_gain: f32,      // qe/tick ganado al defectar
    detection_probability: f32,
    penalty: f32,
) -> f32 {
    let expected_defection = defection_gain - detection_probability * penalty;
    member_benefit - expected_defection  // positivo → cumplir es mejor
}

/// Eficiencia del enforcement: qe recuperado de defectores vs. costo de detección.
pub fn enforcement_efficiency(
    penalty_collected: f32,
    enforcement_cost: f32,
) -> f32 {
    if enforcement_cost <= 0.0 { return 0.0; }
    (penalty_collected - enforcement_cost) / enforcement_cost
}

/// ROI de fundar una institución.
pub fn institution_roi(
    surplus_per_tick: f32,    // beneficio colectivo generado
    member_count: u16,
    admin_cost_per_tick: f32,
    founding_cost: f32,
    horizon_ticks: u32,
) -> f32 {
    let total_benefit = surplus_per_tick * horizon_ticks as f32;
    let total_cost    = founding_cost + admin_cost_per_tick * horizon_ticks as f32;
    total_benefit - total_cost
}

/// Distribución de surplus institucional (proporcional a contribución).
pub fn allocation_share(
    own_contribution: f32,
    total_contributions: f32,
) -> f32 {
    if total_contributions <= 0.0 { return 0.0; }
    own_contribution / total_contributions
}
```

### ET-14B: Tipos

```rust
// src/simulation/emergence/institutions.rs

/// Componente: un agente que pertenece a una institución.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct InstitutionMember {
    pub institution_id: u32,
    pub contribution:   f32,   // qe/tick aportado
    pub compliance:     u8,    // 0=defector, 1=compliant, 2=enforcer
    pub join_tick:      u64,
}

/// Resource: catálogo de instituciones activas.
#[derive(Resource, Default, Debug)]
pub struct InstitutionRegistry {
    pub entries: Vec<InstitutionEntry>,  // sorted by institution_id
}

#[derive(Debug, Clone)]
pub struct InstitutionEntry {
    pub institution_id:    u32,
    pub rule_hash:         u32,    // hash de la regla que define la institución
    pub member_count:      u16,
    pub compliance_rate:   f32,
    pub total_surplus:     f32,    // qe/tick generado por la institución
    pub admin_cost:        f32,
    pub founded_tick:      u64,
}

/// Evento: cambio en membresía o estado de institución.
#[derive(Event, Debug, Clone)]
pub struct InstitutionEvent {
    pub institution_id: u32,
    pub event_type:     InstitutionEventType,
    pub entity:         Entity,
    pub tick_id:        u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum InstitutionEventType { Founded, Joined, Defected, Enforced, Dissolved }
```

### ET-14C: Sistemas

```rust
/// Evalúa estabilidad de instituciones y disuelve las no viables.
/// Phase::MorphologicalLayer — after coalitions (ET-8).
pub fn institution_stability_system(
    mut registry: ResMut<InstitutionRegistry>,
    members: Query<&InstitutionMember>,
    mut events: EventWriter<InstitutionEvent>,
    clock: Res<SimulationClock>,
    config: Res<InstitutionConfig>,
) {
    if clock.tick_id % config.eval_interval as u64 != 0 { return; }

    for entry in registry.entries.iter_mut() {
        // Calcular compliance_rate real
        let total = members.iter().filter(|m| m.institution_id == entry.institution_id).count();
        let compliant = members.iter().filter(|m| {
            m.institution_id == entry.institution_id && m.compliance >= 1
        }).count();
        entry.compliance_rate = if total > 0 { compliant as f32 / total as f32 } else { 0.0 };
        entry.member_count = total as u16;

        let stability = institution_eq::institution_stability(
            entry.compliance_rate,
            config.enforcement_efficiency,
            entry.admin_cost,
        );
        if stability < 0.0 && entry.member_count == 0 {
            events.send(InstitutionEvent {
                institution_id: entry.institution_id,
                event_type: InstitutionEventType::Dissolved,
                entity: Entity::PLACEHOLDER,
                tick_id: clock.tick_id,
            });
        }
    }
    // Limpiar instituciones disueltas
    registry.entries.retain(|e| e.compliance_rate > 0.0 || e.member_count > 0);
}

/// Distribuye surplus institucional a miembros compliant.
/// Phase::MorphologicalLayer — after institution_stability_system.
pub fn institution_surplus_distribution_system(
    mut members: Query<(&mut BaseEnergy, &InstitutionMember)>,
    registry: Res<InstitutionRegistry>,
    config: Res<InstitutionConfig>,
) {
    for entry in &registry.entries {
        let total_contrib: f32 = members.iter()
            .filter(|(_, m)| m.institution_id == entry.institution_id && m.compliance >= 1)
            .map(|(_, m)| m.contribution)
            .sum();

        for (mut energy, member) in &mut members {
            if member.institution_id != entry.institution_id { continue; }
            if member.compliance == 0 { continue; }  // defectores no reciben surplus

            let share = institution_eq::allocation_share(member.contribution, total_contrib);
            let bonus = entry.total_surplus * share;
            let new_qe = (energy.qe() + bonus - config.membership_cost).max(0.0);
            if energy.qe() != new_qe { energy.set_qe(new_qe); }
        }
    }
}
```

### ET-14D: Constantes

```rust
pub struct InstitutionBridge;
impl BridgeKind for InstitutionBridge {}

pub const INSTITUTION_EVAL_INTERVAL:         u8  = 20;
pub const INSTITUTION_ENFORCEMENT_EFFICIENCY: f32 = 0.5;
pub const INSTITUTION_DEFAULT_ADMIN_COST:     f32 = 2.0;  // qe/tick
pub const INSTITUTION_MEMBERSHIP_COST:        f32 = 0.5;  // qe/tick por pertenecer
```

---

## Tacticas

- **Instituciones como Resource, no como entidades.** `InstitutionRegistry` es un Resource porque las instituciones no tienen posición, energía, ni forma — son reglas puras. Sin ECS overhead por entidades vacías.
- **`InstitutionMember` como SparseSet.** La mayoría de entidades no pertenece a ninguna institución en etapas tempranas.
- **Inheritance cultural (ET-3 → ET-14).** Las instituciones formalizan memes exitosos: `rule_hash` es el `behavior_hash` del meme que las fundó. La jerarquía cultural→institucional emerge sin programarla.
- **`InstitutionBridge` cachea surplus_distribution.** Para miembros con misma `(compliance_band, contribution_band)`, la distribución es igual. Hit rate ~75%.
- **Trasciende individuos.** Cuando todos los fundadores mueren, la institución sigue (reglas persisten en `InstitutionRegistry`). Los nuevos miembros heredan las reglas.

---

## NO hace

- No implementa constituciones escritas — `rule_hash` es suficiente como identificador de regla.
- No modela democracia/votación — el liderazgo es el miembro con mayor contribución.
- No persiste instituciones entre sesiones de juego — `InstitutionRegistry` se reinicia.

---

## Dependencias

- ET-3 `CulturalMemory` — memes se formalizan en `rule_hash` de institución.
- ET-8 `CoalitionRegistry` — coaliciones estables son el núcleo fundacional.
- ET-4 `InfrastructureInvestEvent` — las instituciones coordinan inversión colectiva via este evento.
- ET-11 `MultiscaleSignalGrid` — `ms.global` indica escasez → presión a fundar institución redistributiva.

---

## Criterios de Aceptación

- `institution_stability(0.8, 0.5, 0.2)` → `0.2`.
- `institution_stability(0.2, 0.5, 0.5)` → `-0.4` (institución no viable).
- `compliance_incentive(10.0, 8.0, 0.5, 20.0)` → `8.0` (cumplir mejor).
- `compliance_incentive(10.0, 15.0, 0.1, 1.0)` → `-4.9` (defectar mejor).
- `allocation_share(5.0, 20.0)` → `0.25`.
- Test: institución con compliance_rate → 0 → `InstitutionEventType::Dissolved` emitido.
- Test: miembro compliant en institución con surplus → qe aumenta.
- Test: defector en institución → no recibe surplus.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-3 Cultural Transmission — memes que se formalizan
- ET-8 Dynamic Coalitions — grupos que se institucionalizan
- ET-4 Infrastructure — canal de inversión coordinada
- Blueprint §T4-1: "Institutions", collective coordination mechanics
