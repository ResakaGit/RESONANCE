# ADR-020: Particle Infrastructure — Spatial Acceleration, Precision, Scale

**Estado:** Implementado
**Fecha:** 2026-04-13
**Contexto:** PARTICLE_CHARGE track, sprints PC-0/1/2

## Contexto

El batch particle simulator tiene fuerzas O(N^2) limitadas a 128 particulas con
acumulacion en f32. Para atomos emergentes necesitamos: mas particulas, mejor
precision, y fuerzas que escalen.

## Decisiones implementadas

### D1: NO escalar SimWorldFlat — particle_lab es independiente

**Problema original:** PC-0 proponia aumentar MAX_ENTITIES de 128 a 1024 en
`SimWorldFlat`, inflando memoria para TODOS los experiments (cancer, fermi, etc.).

**Descubrimiento:** `particle_lab.rs` ya tiene su propio loop con `ChargedParticle`
array — no usa `SimWorldFlat`. Escalar la ecologia es innecesario y costoso.

**Decision:** Escalar solo `particle_lab.rs` de `[ChargedParticle; 128]` a
`Vec<ChargedParticle>` (capacidad dinamica). `positive_count`/`negative_count`
de u8 a u16. SimWorldFlat intacto.

### D2: Barnes-Hut quadtree — solo Coulomb aproximado

**Problema:** O(N^2) con N=1024 = ~500K pares por tick.

**Decision:** QuadTree en `equations/spatial_tree.rs`. Pure math, sin Bevy.

**Precision critica:** Solo Coulomb (1/r^2, long-range) usa la aproximacion
del tree. LJ (1/r^12, short-range) se computa exactamente en interacciones
leaf-leaf. Aproximar LJ con centroide da >700% de error porque la funcion
es extremadamente sensible a distancia corta.

**Parametros numericos (NO derivados de constantes fisicas):**
- `THETA = 0.5` — opening angle standard de Barnes-Hut
- `BRUTE_FORCE_THRESHOLD = 64` — crossover empirico tree vs brute

Estos NO se derivan de KLEIBER ni DENSITY_SCALE. Son parametros numericos
del algoritmo, no fisica. Derivarlos forzadamente seria deshonesto.

### D3: Acumulacion interna en f64

**Problema:** f32 tiene ~7 digitos de precision. Acumular ~1000 fuerzas
pequeñas causa cancelacion catastrofica.

**Decision:** `accumulate_brute_f64` acumula en `[f64; 2]`, cast a f32 solo
al final. `accumulate_forces` ahora delega a `spatial_tree::accumulate_forces_adaptive`
que usa f64 internamente en ambos paths (brute y tree).

**Impacto:** Zero para el caller. La API retorna `Vec<[f32; 2]>` como antes
(cambiado de `[[f32; 2]; 128]` fijo a Vec dinamico).

### D4: Damping derivado de DISSIPATION_SOLID

**Problema:** `damping = 0.005` hardcoded inline en `particle_lab.rs:99`.
Viola coding rule 10 (constants centralized) y 12 (no inline formulas).

**Decision:** Reemplazado por `DISSIPATION_SOLID` (que es exactamente 0.005).
Axiom 4: la disipacion de particulas usa la misma constante fundamental
que toda la simulacion. No es coincidencia — es el mismo fenomeno fisico.

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `blueprint/equations/spatial_tree.rs` | **NUEVO** — Barnes-Hut quadtree, 7 tests |
| `blueprint/equations/coulomb.rs` | `accumulate_forces` → dispatch adaptivo via spatial_tree |
| `blueprint/equations/mod.rs` | +`pub mod spatial_tree` |
| `use_cases/experiments/particle_lab.rs` | Vec<ChargedParticle>, u16 counts, damping→DISSIPATION_SOLID |
| `batch/systems/particle_forces.rs` | Adaptado a Vec return de accumulate_forces |
| `bin/particle_lab.rs` | u8→u16 casts |

## No viola axiomas

- Axioma 5: Newton 3 preservado (brute: simetrico, tree: leaf-leaf exacto)
- Axioma 7: 1/r^2 preservado (tree solo aproxima Coulomb, no LJ)
- Axioma 4: damping derivado de DISSIPATION_SOLID (no hardcoded)
- Axioma 8: frequency alignment no afectada (se calcula en bond_energy, no en force)

## Tests

- 7 tests spatial_tree (empty, single, two-vs-brute, newton3, 100-particle RMS, dispatch, determinism)
- 26 tests coulomb (todos preexistentes, pasan sin cambios)
