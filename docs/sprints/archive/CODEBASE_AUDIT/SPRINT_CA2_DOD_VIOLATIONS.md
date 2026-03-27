# Sprint CA-2 — Fix violaciones DOD

**Módulo:** `src/layers/`, `src/entities/`
**Tipo:** Refactor acotado. Reemplazar tipos heap por stack en componentes + eliminar `.expect()` en producción.
**Onda:** 1 — Requiere CA-1 (compilación verde).
**Estado:** ✅ Cerrado (2026-03-25) — 4/5 fixes aplicados. `attention_gating_system` pendiente decisión

## Objetivo

Corregir 4 violaciones confirmadas de Hard Blocks y DOD. Cada fix es independiente y puede hacerse en cualquier orden.

## Diagnóstico

| # | Componente | Violación | Archivo | Línea | Severidad |
|---|------------|-----------|---------|-------|-----------|
| 1 | `MobaIdentity` (L9) | `relational_tags: Vec<RelationalTag>` — heap alloc en componente | `layers/identity.rs` | 41 | Media |
| 2 | `AlchemicalForge` | `mastered_elements: Vec<ElementId>` + `discovered_compounds: Vec<ElementId>` | `layers/engine.rs` | 156, 162 | Media |
| 3 | `AbilitySlot` (dentro de `Grimoire`) | `name: String` — HB #6 | `layers/will.rs` | 172 | Media |
| 4 | `EntityBuilder` | `.expect()` en producción — HB #10 | `entities/builder.rs` | 205 | Alta |

## Pasos de implementación

### CA-2A: `MobaIdentity.relational_tags` → bitmask `u16`

**Contexto:** `RelationalTag` es un enum cerrado. Si tiene ≤16 variantes, cabe en un `u16` bitmask.

1. Contar variantes de `RelationalTag`.
2. Si ≤16:
   ```rust
   pub struct MobaIdentity {
       pub(crate) faction: Faction,
       pub(crate) relational_tags: u16,  // bitmask
       pub(crate) critical_multiplier: f32,
   }

   impl MobaIdentity {
       pub fn has_tag(&self, tag: RelationalTag) -> bool {
           (self.relational_tags & (1 << tag as u16)) != 0
       }
       pub fn add_tag(&mut self, tag: RelationalTag) {
           self.relational_tags |= 1 << tag as u16;
       }
       pub fn remove_tag(&mut self, tag: RelationalTag) {
           self.relational_tags &= !(1 << tag as u16);
       }
   }
   ```
3. Si >16 y ≤32: usar `u32`. Si >32: stack-allocated `[RelationalTag; N]` con count.
4. Actualizar todos los call sites (`has_tag`, `push`, `contains`, `iter`).
5. `cargo test --lib` verde.

### CA-2B: `AlchemicalForge` → arrays fijos

**Contexto:** Cantidad de elementos maestrizados y compuestos descubiertos está naturalmente acotada.

1. Determinar cota superior realista (probablemente ≤8 por el sistema de elementos).
2. Reemplazar:
   ```rust
   pub struct AlchemicalForge {
       pub mastered: [ElementId; 8],
       pub mastered_count: u8,
       pub creation_bonus: f32,
       pub discovered: [ElementId; 8],
       pub discovered_count: u8,
   }
   ```
3. **Nota:** Esto son 5 campos (>4 regla). Alternativa: partir en `AlchemicalForge` (2 campos: `primary_element`, `creation_bonus`) + `AlchemicalDiscovery` componente separado (3 campos: arrays + count). Evaluar si la partición tiene sentido semántico.
4. Agregar métodos `mastered_slice()`, `discover()`, `has_mastered()`.
5. Actualizar call sites.
6. `cargo test --lib` verde.

### CA-2C: `AbilitySlot.name` → `&'static str`

**Contexto:** Los nombres de abilities son constantes conocidas en compile-time o definidas en presets RON.

1. Evaluar de dónde vienen los nombres:
   - Si hardcoded → `&'static str` directo.
   - Si deserializados de RON → necesitan `String` en el loader, luego internar a `&'static str` via `Box::leak` (aceptable en startup) o usar `u16` ID con registry Resource.
2. Opción preferida (si nombres son pocos y fijos):
   ```rust
   pub struct AbilitySlot {
       pub name: &'static str,
       pub output: AbilityOutput,
       pub cast: AbilityCastSpec,
   }
   ```
3. Si `Deserialize` necesita `String`:
   ```rust
   pub struct AbilitySlot {
       pub name_id: u16,
       pub output: AbilityOutput,
       pub cast: AbilityCastSpec,
   }

   #[derive(Resource)]
   pub struct AbilityNameRegistry(pub Vec<&'static str>);
   ```
4. Actualizar `Grimoire` y call sites.
5. `cargo test --lib` verde.

### CA-2D: `EntityBuilder.expect()` → `let-else`

**Fix directo:**

```rust
// Antes (línea 201-207):
pub fn with_metabolic_graph_inferred(mut self, t_core: f32, t_env: f32) -> Self {
    let manifest = self
        .organ_manifest
        .as_ref()
        .expect("with_organ_manifest must be called before with_metabolic_graph_inferred");
    self.metabolic_graph = Some(equations::metabolic_graph_from_manifest(manifest, t_core, t_env));
    self
}

// Después:
pub fn with_metabolic_graph_inferred(mut self, t_core: f32, t_env: f32) -> Self {
    let Some(manifest) = self.organ_manifest.as_ref() else { return self; };
    self.metabolic_graph = Some(equations::metabolic_graph_from_manifest(manifest, t_core, t_env));
    self
}
```

**Decisión de diseño:** ¿Silenciar el error (return self) o propagar? En un builder fluent, el patrón standard es skip silencioso + `debug_assert!` opcional:
```rust
debug_assert!(self.organ_manifest.is_some(),
    "with_organ_manifest must be called before with_metabolic_graph_inferred");
let Some(manifest) = self.organ_manifest.as_ref() else { return self; };
```

## Tácticas

- **Un fix, un commit.** Cada sub-tarea es atómica.
- **Buscar call sites antes de cambiar la API.** `grep -r "relational_tags\|has_tag\|add_tag"` etc.
- **Preservar semántica.** El bitmask debe comportarse idéntico al Vec para todos los usos actuales.
- **Si AlchemicalForge >4 campos → partir.** No violar la regla DOD para resolver otra violación.

## NO hace

- No agrega funcionalidad nueva.
- No modifica sistemas — solo componentes y el builder.
- No toca ecuaciones ni constantes.

## DoD

- 0 violaciones DOD en los archivos afectados.
- Todos los componentes modificados: ≤4 campos, sin `Vec`, sin `String`, sin `Box<dyn>`.
- `EntityBuilder` sin `.expect()` en ningún método.
- `cargo test --lib` verde.
- `cargo clippy` sin warnings nuevos en archivos modificados.

## Referencias

- `CLAUDE.md` — Hard Blocks #6 (no String), #10 (no expect)
- `docs/sprints/SPRINT_PRIMER.md` — Regla: max 4 campos por componente
- `src/layers/identity.rs` — MobaIdentity
- `src/layers/engine.rs` — AlchemicalForge
- `src/layers/will.rs` — AbilitySlot / Grimoire
- `src/entities/builder.rs` — EntityBuilder
