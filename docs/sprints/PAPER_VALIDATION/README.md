# Track: PAPER_VALIDATION — Validación contra literatura publicada

**Objetivo:** Implementar 5 experimentos nuevos que comparen predicciones de RESONANCE contra datos cuantitativos de papers publicados peer-reviewed. Cada experimento es un módulo stateless (`Config → Report`) + binary dedicado + tests BDD. Zero acoplamiento con código existente.

**Estado:** PENDIENTE
**Bloqueado por:** Nada (usa batch/ y equations/ existentes, no los modifica)
**Desbloquea:** Paper v2 (multi-comparator), partnerships pharma, credibility model upgrade

---

## Principio de diseño: ZERO acoplamiento

```
NO modifica:
  - src/blueprint/equations/*.rs (usa, no cambia)
  - src/batch/systems/*.rs (usa, no cambia)
  - src/simulation/*.rs (no lo toca)
  - src/use_cases/experiments/pathway_inhibitor_exp.rs (no lo toca)
  - src/use_cases/experiments/cancer_therapy.rs (no lo toca)

SÍ crea (todo nuevo):
  - src/use_cases/experiments/paper_zhang2022.rs     (PV-1)
  - src/use_cases/experiments/paper_sharma2010.rs    (PV-2)
  - src/use_cases/experiments/paper_hill_ccle.rs     (PV-3)
  - src/use_cases/experiments/paper_foo_michor2009.rs (PV-4)
  - src/use_cases/experiments/paper_michor2005.rs    (PV-5)
  - src/bin/paper_validation.rs                      (runner unificado)
  - 1 línea en src/use_cases/experiments/mod.rs      (5 pub mod)
```

Patrón obligatorio por experimento:
```rust
// Config pura (no depende de estado global)
pub struct PaperXConfig { ... }
impl Default for PaperXConfig { ... }  // valores del paper

// Report puro (datos, no side effects)
pub struct PaperXReport { ... }

// Función pura: config → report
pub fn run_paper_x(config: &PaperXConfig) -> PaperXReport { ... }

// Tests BDD inline
#[cfg(test)]
mod tests { ... }
```

---

## 5 Sprints

| Sprint | Paper | Dato cuantitativo clave | Dificultad |
|--------|-------|------------------------|------------|
| [PV-1](SPRINT_PV1_ZHANG_ADAPTIVE.md) | Zhang et al. 2022 (eLife) | TTP 33.5 vs 14.3 meses, Lotka-Volterra params | Medio |
| [PV-2](SPRINT_PV2_SHARMA_PERSISTERS.md) | Sharma et al. 2010 (Cell) | 0.3% persisters, 100x resistencia, 9 doublings recovery | Medio |
| [PV-3](SPRINT_PV3_HILL_CALIBRATION.md) | GDSC/CCLE | Distribución real de Hill slopes vs n=2 assumption | Bajo |
| [PV-4](SPRINT_PV4_FOO_PULSED.md) | Foo & Michor 2009 (PLoS) | P(resistencia) = 1-exp(-uB), continuous vs pulsed | Medio |
| [PV-5](SPRINT_PV5_MICHOR_BIPHASIC.md) | Michor et al. 2005 (Nature) | Biphasic CML decline slopes 0.05 vs 0.005/day | Medio-Alto |

---

## Dependencias

```
PV-1 ──┐
PV-2 ──┤
PV-3 ──┼── todos independientes (paralelos)
PV-4 ──┤
PV-5 ──┘
         └──→ bin/paper_validation.rs (runner unificado, post todos)
```

Todos los sprints pueden ejecutarse en paralelo. El runner unificado se crea al final.

---

## Criterios de cierre del track

- [ ] 5 módulos en `src/use_cases/experiments/paper_*.rs`
- [ ] Cada módulo: Config + Report + run() + ≥5 tests BDD
- [ ] `cargo run --release --bin paper_validation` ejecuta los 5 y reporta PASS/FAIL por paper
- [ ] 0 modificaciones a archivos existentes (excepto 1 línea en mod.rs + Cargo.toml [[bin]])
- [ ] Cada test tiene comentario citando el dato exacto del paper (autor, tabla, página)
- [ ] README de cada sprint documenta: paper DOI, dato extraído, mapeo a RESONANCE, resultado esperado
