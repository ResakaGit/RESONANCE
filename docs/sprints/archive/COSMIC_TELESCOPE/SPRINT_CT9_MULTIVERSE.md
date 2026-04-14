# CT-9: Multiverse Seeds — Múltiples realidades, un mismo cluster

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** CT-1
**ADR:** ADR-036 §D7

## Objetivo

Cada zoom es un branch del multiverso. El mismo cluster con distinto seed
produce una realidad distinta pero termodinámicamente válida. El usuario puede
comparar realidades y observar la probabilidad emergente de fenómenos como vida.

## Precondiciones

- CT-1 completado (zoom engine con seed branching)

## Entregables

### E1: `multiverse.rs` — registro de branches

```rust
// src/cosmic/multiverse.rs

/// Un branch del multiverso: un zoom con seed específico.
#[derive(Clone, Debug)]
pub struct MultiverseBranch {
    pub parent_entity_id: u32,
    pub scale: ScaleLevel,
    pub seed: u64,
    pub timestamp: u64,          // tick del universo al momento del zoom
    pub snapshot: BranchSnapshot, // métricas al momento de zoom-out
}

pub struct BranchSnapshot {
    pub total_qe: f64,
    pub n_entities: usize,
    pub has_life: bool,          // ¿abiogenesis produjo vida?
    pub max_q_folding: f64,      // mejor Q de proteína observada
    pub species_count: usize,
}

#[derive(Resource, Default)]
pub struct MultiverseLog {
    pub branches: Vec<MultiverseBranch>,
}

impl MultiverseLog {
    /// Probabilidad de vida: branches con vida / total branches en mismo cluster.
    pub fn life_probability(&self, parent_id: u32) -> f64;

    /// Resumen estadístico de todas las realidades visitadas.
    pub fn summary(&self) -> MultiverseSummary;
}
```

### E2: Tab → cycle seed

En el binario `cosmic_telescope`:
- Tab → incrementa seed → re-colapsa nivel actual con nuevo seed
- El nivel se regenera con nueva realidad
- Branch anterior se guarda en MultiverseLog

### E3: Comparison view

Modo especial (tecla C) que muestra 2 realidades side-by-side:
- Izquierda: realidad actual
- Derecha: última realidad visitada del mismo cluster
- Métricas comparativas: qe, entities, life, Q

### E4: Probabilistic observables

Después de visitar N branches del mismo cluster:
- `life_probability(cluster_id)` = ¿con qué frecuencia emerge vida?
- `mean_species_count` = diversidad promedio
- `mean_folding_Q` = calidad de folding promedio

Esto es un resultado científico emergente: la probabilidad de vida es un
observable del sistema, no un parámetro.

## Tasks

- [ ] Crear `src/cosmic/multiverse.rs`
- [ ] `MultiverseLog`: registro de branches visitados
- [ ] `life_probability`: cálculo estadístico
- [ ] Input: Tab → cycle seed → re-collapse
- [ ] Comparison view (split screen)
- [ ] Probabilistic summary en HUD
- [ ] Tests:
  - `same_seed_same_branch` (determinismo)
  - `different_seeds_different_branches`
  - `life_probability_between_0_and_1`
  - `log_accumulates_branches`
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Tab produce nueva realidad determinista
2. MultiverseLog acumula branches correctamente
3. `life_probability` converge con más samples
4. Comparison view muestra 2 realidades legibles
5. Métricas son observables emergentes (no hardcoded thresholds)
