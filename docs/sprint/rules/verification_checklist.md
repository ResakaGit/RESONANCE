# Verification Checklist (Pre-Merge)

Cada PR con systems nuevos pasa por este checklist. Rol: **Verificador**.

---

## 1. Contrato Módulo/Phase

- [ ] System asignado a Phase correcta (ver `design_rules.md` R-5)
- [ ] Si emite eventos → consumidor registrado en `.chain()` o `.after()`
- [ ] Si lee output de otro system → Phase posterior o dependencia explícita
- [ ] System registrado en el Plugin correcto (SimulationPlugin, DebugPlugin, etc.)
- [ ] Event registrado en `bootstrap.rs` (si es nuevo)

## 2. Correctitud Matemática

- [ ] Toda fórmula vive en `blueprint/equations/{dominio}/mod.rs`
- [ ] Ecuación tiene test unitario con edge cases: zero, negative, max, NaN guard
- [ ] Constantes extraídas a `blueprint/constants/{dominio}.rs`
- [ ] Resultado de ecuación acotado (no puede ser NaN, Inf, negativo si no tiene sentido)
- [ ] `finite_non_negative()` o equivalente aplicado donde corresponde
- [ ] Ecuación es **dimensionalmente consistente** (units check: qe/s, Hz, m, etc.)

## 3. ECS / DOD Integrity

- [ ] Ningún componente tiene >4 fields
- [ ] Ningún system lee >5 component types
- [ ] No hay `Box<dyn Trait>` en components — solo enums
- [ ] No hay `String` en components — solo enums, u32, &'static str
- [ ] No hay `#[derive(Bundle)]` — solo tuples
- [ ] No hay `unsafe`
- [ ] No hay `unwrap()`/`expect()`/`panic!()` en systems
- [ ] `Res` usado donde `ResMut` no es necesario
- [ ] Change detection guard en toda mutación: `if val != new { val = new; }`
- [ ] `SparseSet` storage para componentes transient

## 4. Determinismo

- [ ] Sin RNG no-determinista (si necesitas RNG, usar seed del Resource)
- [ ] Sin dependencia en orden de iteración de HashMap
- [ ] Sin floating-point comparison exacta (usar epsilon)
- [ ] Mismo input → mismo output garantizado
- [ ] Cursor-based iteration es round-robin (no random start)

## 5. Performance Hot Path

- [ ] Sin `Vec::new()` dentro de loops de entidades
- [ ] Sin `HashMap` en hot path — usar sorted Vec o SpatialIndex
- [ ] Sin `clone()` de structs grandes en loops
- [ ] Spatial queries usan `SpatialIndex` existente (no brute force)
- [ ] Si N² → throttled con cursor + MAX_PER_FRAME budget
- [ ] `Local<Vec<T>>` para scratch buffers
- [ ] No polling innecesario — `run_if` o `Changed<T>` filters

## 6. Test Evidence

- [ ] Ecuaciones: `#[cfg(test)] mod tests` en mismo archivo
- [ ] System: test de integración con `MinimalPlugins`, 1 update, assert delta
- [ ] Edge case: entity sin componente opcional → no panic
- [ ] Edge case: 0 entidades → no crash
- [ ] Edge case: 1 entidad → correcto
- [ ] Edge case: valores extremos (qe=0, radius=MIN, frequency=0)
- [ ] Naming: `{function}_{condition}_{expected_result}`

## 7. Integration

- [ ] Componentes nuevos registrados en `LayersPlugin` (`.register_type::<T>()`)
- [ ] Resources nuevos inicializados en `bootstrap.rs`
- [ ] Nuevo system aparece en `pipeline.rs` en su Phase correcta
- [ ] Si nuevo Event → registrado en `bootstrap.rs`
- [ ] Si nuevo spawn function → en `archetypes.rs`
- [ ] Si nueva constante → facade `constants/mod.rs` la re-exporta
- [ ] Si nueva ecuación → facade `equations/mod.rs` la re-exporta

## Veredicto

| Resultado | Criterio |
|-----------|----------|
| **PASS** | Todo checked. Merge. |
| **WARN** | 1-2 items menores (docs, naming). Merge con fix pendiente. |
| **BLOCK** | Cualquier falla en §2 (math), §3 (ECS), §4 (determinismo). No merge. |
