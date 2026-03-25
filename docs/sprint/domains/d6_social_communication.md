# D6: Social & Communication

**Prioridad**: P2
**Phase**: `Phase::MetabolicLayer` (después de trophic)
**Dependencias**: D1 (Behavior), D5 (Sensory), L13 (StructuralLink), events
**Systems**: 3

---

## Motivación Científica

El comportamiento social emerge cuando la cooperación beneficia la fitness individual (Hamilton 1964: rb > c, donde r=relatedness, b=benefit, c=cost). Manadas de lobos cazan presas más grandes que individuos. Bandadas de pájaros reducen predation risk.

En Resonance, las relaciones sociales se modelan via **L13 StructuralLink** (spring joints como "lazos sociales") y **L9 MobaIdentity** (faction + tags). La "manada" es un grupo de entidades conectadas por StructuralLinks blandos (low stiffness, high rest_length).

---

## Componentes Nuevos

### C1: PackMembership (3 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
pub struct PackMembership {
    pub pack_id: u32,
    pub role: PackRole,
    pub joined_tick: u32,
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum PackRole {
    Leader,
    Member,
    Juvenile,
}
```

### C2: SocialBond (2 fields, vía L13)
Reusar StructuralLink con semántica social:
- `stiffness` baja (0.01) = lazo social flexible
- `rest_length` alto (5.0-10.0) = distancia de grupo
- `break_stress` medio (50.0) = lazo se rompe si se separan mucho

---

## Ecuaciones Nuevas

### E1: `pack_cohesion_force(member_pos: Vec2, centroid: Vec2, rest_distance: f32) -> Vec2`
```
direction = (centroid - member_pos).normalize_or_zero()
distance = (centroid - member_pos).length()
force = COHESION_STRENGTH × (distance - rest_distance) × direction
```

### E2: `dominance_contest_score(qe: f32, radius: f32, resilience: f32) -> f32`
```
score = qe × radius × (1 + resilience × DOMINANCE_RESILIENCE_WEIGHT)
```

### E3: `pack_hunt_bonus(pack_size: u32, prey_qe: f32) -> f32`
```
bonus = (pack_size as f32).sqrt() × COOPERATIVE_HUNT_SCALE
// Diminishing returns: 2 lobos = 1.4×, 4 lobos = 2×, 9 lobos = 3×
```

---

## Systems (3)

### S1: `social_pack_formation_system` (Transformer)
**Phase**: MetabolicLayer
**Reads**: Transform, MobaIdentity (same faction), TrophicRole, SpatialIndex
**Writes**: PackMembership, StructuralLink (create social bonds)
**Run condition**: Every 16 ticks
**Logic**:
1. Find unattached social entities (no PackMembership) near other same-faction entities
2. If 2+ within PACK_FORMATION_RADIUS → create pack (lowest Entity.index() = Leader)
3. Insert StructuralLink between members (soft spring)
4. Insert PackMembership on each

### S2: `social_pack_cohesion_system` (Transformer)
**Phase**: MetabolicLayer, after S1
**Reads**: PackMembership, Transform, StructuralLink
**Writes**: WillActuator (add cohesion force to movement_intent)
**Logic**: Miembros se mueven hacia centroid del pack. Leader decide dirección; members siguen.

### S3: `social_dominance_system` (Transformer)
**Phase**: MetabolicLayer, after S2
**Reads**: PackMembership, BaseEnergy, SpatialVolume, InferenceProfile
**Writes**: PackMembership (role changes)
**Run condition**: Every 60 ticks
**Logic**:
1. Per pack: compute dominance_score for each member
2. Highest score = Leader
3. If leader changed → update roles

---

## Tests

- `pack_forms_when_two_entities_near`
- `pack_leader_is_strongest`
- `pack_cohesion_moves_toward_centroid`
- `pack_hunt_bonus_scales_with_sqrt_size`
- `social_bond_breaks_when_too_far`
