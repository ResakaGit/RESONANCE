# ADR-025: Ewald vs Cutoff for Go Models — Decision Gate

**Estado:** Pendiente (decidir despues de MD-9)
**Fecha:** 2026-04-13
**Contexto:** MOLECULAR_DYNAMICS track, sprints [MD-9](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD9_PEPTIDE.md) (gate), [MD-12](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD12_EWALD.md), Risk R6

## Contexto

Ewald summation (MD-12) resuelve la interaccion Coulomb de largo alcance en
sistemas periodicos. Es necesaria para simulaciones con cargas explicitas
(proteinas en agua). Pero es compleja (3 semanas de esfuerzo, O(N^{3/2})).

## Pregunta

El Go model (MD-15) usa contactos nativos de corto alcance (< 8 A). Las
fuerzas son fundamentalmente LJ-like, no Coulomb. ¿Necesita Ewald?

## Opciones

### Opcion A: Implementar Ewald (3 semanas)

**Pro:** Completo. Permite Phase 2 (solvated peptide). Necesario si se quiere
ir mas alla de Go models.

**Contra:** 3 semanas de esfuerzo para algo que el Go model no necesita.

### Opcion B: Cutoff Coulomb + reaction field (3 dias)

```rust
V_rf(r) = q_i * q_j * [1/r + r^2/(2*r_cut^3) - 3/(2*r_cut)]
```

**Pro:** Simple. Suficiente para Go models. Permite avanzar directo a Phase 3.

**Contra:** No soporta solvated peptide (TIP3P necesita Ewald para estabilidad).

### Opcion C: Skip (0 esfuerzo)

**Pro:** Maximo shortcut.

**Contra:** Si el Go model necesita Coulomb (cargas en sidechains), no funciona.

## Decision Gate

Despues de MD-9 (peptide in vacuum), evaluar:

1. ¿El Go model necesita Coulomb? → Si no, Opcion C (skip).
2. ¿Se quiere Phase 2 (water)? → Si si, Opcion A (Ewald).
3. ¿Solo Go model sin agua? → Opcion B (reaction field).

**Recomendacion provisional:** Opcion B. Reaction field es suficiente para
el shortcut path. Ewald se implementa solo si Phase 2 se activa.

## Consecuencias

- Si Opcion B: MD-12 se reduce de 3 semanas a 3 dias
- Si Opcion C: MD-12 se cancela, Phase 2 se cancela
- En todos los casos: MD-15 (Go model) procede sin bloqueo
