# Sprint G8 — Change Detection Guards (`set_if_neq`)

**Tipo:** Refactor — corregir falsos positivos de change detection.
**Riesgo:** BAJO — solo agrega guards antes de mutaciones.
**Onda:** 0 — Sin dependencias. Aplicable gradualmente.
**Estado:** Pendiente

## Objetivo

Eliminar falsos positivos de `Changed<T>` en todo el codebase. Actualmente, acceder `&mut Component` marca el componente como changed incluso si el valor no cambia. Esto causa que sistemas con `Changed<T>` filters se ejecuten innecesariamente.

## Estado actual en Resonance

- Solo 2 archivos usan `is_changed()`
- Ningun archivo usa `set_if_neq`
- Sistemas de worldgen visual usan `Changed<T>` pero no guardan las mutaciones
- Multiples sistemas mutan componentes sin verificar si el valor cambio

## Responsabilidades

### Paso 1 — Auditar sistemas que mutan componentes

Buscar todos los patrones:
```rust
// Patron problematico: &mut sin guard
for mut component in &mut query {
    component.field = new_value;  // marca changed siempre
}
```

Archivos clave a revisar:
- `src/simulation/` — todos los sistemas de gameplay
- `src/worldgen/systems/` — sistemas de worldgen
- `src/eco/systems.rs` — eco boundaries

### Paso 2 — Aplicar guards

Para cada mutacion, agregar verificacion de igualdad:

```rust
// Opcion A: guard manual
if component.field() != new_value {
    component.set_field(new_value);
}

// Opcion B: set_if_neq (Bevy built-in para Mut<T>)
component.set_if_neq(MyComponent::new(new_value));
```

**Preferir Opcion A** cuando se muta un campo individual. **Opcion B** cuando se reemplaza todo el componente.

### Paso 3 — Agregar `set_if_neq` a componentes con setters

Los componentes de Resonance ya tienen setters (`set_qe`, `set_frequency`, etc.). Asegurar que los setters solo muten si el valor cambia:

```rust
impl BaseEnergy {
    pub fn set_qe(&mut self, val: f32) {
        let clamped = val.max(0.0);
        if self.qe != clamped {
            self.qe = clamped;
        }
    }
}
```

**Alternativa:** No modificar setters (mantenerlos simples) y dejar la responsabilidad al sistema que llama. Ambas aproximaciones son validas. Elegir una y ser consistente.

### Paso 4 — Verificar sistemas con `Changed<T>`

Buscar todos los `Changed<T>` en queries y verificar que los productores de cambios usan guards:

| Sistema consumidor | Filter | Productores |
|-------------------|--------|-------------|
| (identificar via grep) | `Changed<BaseEnergy>` | dissipation, collision, catalysis |
| (identificar via grep) | `Changed<Transform>` | movement, pathfinding |
| (identificar via grep) | `Changed<MatterCoherence>` | phase_transition |

## Tacticas

- **Gradual.** Aplicar archivo por archivo. Cada archivo es un commit.
- **Priorizar sistemas con `Changed<T>` consumers.** Si no hay consumidores de `Changed<T>` para un componente, el guard es menos urgente (pero sigue siendo buena practica).
- **No over-engineer.** No crear macro `guard_set!()` — el pattern `if x != y { x = y; }` es claro y simple.

## NO hace

- No modifica logica de ningun sistema.
- No cambia valores calculados.
- No agrega `Changed<T>` filters nuevos — solo arregla los existentes.
- No toca tests (los valores finales son los mismos).

## Criterio de aceptacion

- [ ] Todo sistema que muta un componente verifica igualdad antes de mutar
- [ ] Grep confirma: patron `for mut.*in &mut` siempre tiene guard
- [ ] `Changed<T>` filters no se disparan sin cambio real
- [ ] `cargo check` pasa
- [ ] `cargo test` — 575+ tests pasan (sin regresiones)
- [ ] Performance: menos sistemas triggered por `Changed<T>` falso

## Esfuerzo estimado

~2-3 horas. Mecanico pero requiere revisar cada sistema.
