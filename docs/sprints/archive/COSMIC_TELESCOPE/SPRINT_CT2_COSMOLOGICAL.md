# CT-2: Cosmological Scale — N-Body Gravitacional + Cluster Formation

**Esfuerzo:** L (3–5 sesiones)
**Bloqueado por:** CT-0
**ADR:** ADR-036 §D1 (S0)

## Objetivo

Simular la escala cosmológica: N clusters de energía interactuando
gravitacionalmente. Big Bang = toda la qe concentrada en un punto que se expande.
Clusters se forman por atracción gravitacional modulada por frecuencia (Axiom 8).

## Precondiciones

- CT-0 completado (`ScaleLevel::Cosmological`, `ScaleManager`)
- `TensionField` (L11) con `InverseSquare` falloff
- `verlet.rs` con position/velocity step 3D

## Entregables

### E1: `cosmological.rs` — simulación S0

```rust
// src/cosmic/scales/cosmological.rs

pub struct CosmoConfig {
    pub n_initial_clusters: usize,     // ~100-500
    pub total_qe: f64,                  // energía total del universo
    pub expansion_rate: f64,            // Hubble-like, derivado de dissipation
    pub dt: f64,                        // timestep cosmológico
    pub seed: u64,
}

/// Tick cosmológico: gravedad + expansión + dissipation.
pub fn cosmo_tick(world: &mut SimWorldFlat, config: &CosmoConfig);

/// Inicializar Big Bang: N partículas cerca del origen con velocidades radiales.
pub fn init_big_bang(config: &CosmoConfig) -> SimWorldFlat;
```

**Física:**
- Gravedad: `F = G × m1 × m2 / r²` donde `m = qe` y `G` derivado de `DENSITY_SCALE`
- Expansión: velocidad radial ∝ distancia × `expansion_rate` (Hubble emergente)
- Dissipation: `DISSIPATION_GAS` (el universo temprano es plasma→gas)
- Axiom 8: clusters con frecuencias alineadas se atraen más fuerte
- Axiom 7: atracción decae con distancia (InverseSquare)

**Emergencia esperada:**
- Clusters se forman por colapso gravitacional
- Clusters más masivos (más qe) dominan (Axiom 3: competition)
- Frecuencias se segregan: clusters con freq similar se fusionan

### E2: Binario de validación

```rust
// src/bin/cosmic_bigbang.rs — validación standalone
// Corre Big Bang, reporta: N clusters formados, distribución de masas,
// segregación de frecuencias, conservación total de qe.
```

## Tasks

- [ ] Crear `src/cosmic/scales/cosmological.rs`
- [ ] `init_big_bang`: distribución inicial de partículas
- [ ] `cosmo_tick`: gravedad + expansión + dissipation + Axiom 8
- [ ] Detección de clusters: partículas dentro de radio ← `neighbor_list.rs`
- [ ] Crear `src/bin/cosmic_bigbang.rs` — validación standalone
- [ ] Tests:
  - `big_bang_conserves_total_qe` (con dissipation: qe decrece monotone)
  - `clusters_form_from_uniform_initial` (N clusters < N initial)
  - `expansion_increases_mean_distance`
  - `frequency_segregation_emerges` (clusters más coherentes internamente)
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Big Bang → clusters estables en <10k ticks cosmológicos
2. `total_qe` monotone decreasing (Axiom 5)
3. Clusters más masivos tienen menor freq variance interna (Axiom 8)
4. Gravedad InverseSquare (Axiom 7)
5. Binario reporta métricas reproducibles con mismo seed
