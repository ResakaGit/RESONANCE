# CT-4: Stellar Scale — Cluster → Estrellas + Protoplanetas

**Esfuerzo:** L (3–5 sesiones)
**Bloqueado por:** CT-2
**ADR:** ADR-036 §D4 (S0→S1)

## Objetivo

Al hacer zoom en un cluster cosmológico, expandirlo en estrellas, gas y discos
protoplanetarios. La formación estelar emerge de colapso gravitacional local +
coherencia de frecuencia.

## Precondiciones

- CT-2 completado (escala cosmológica con clusters estables)
- CT-1 completado (zoom engine)

## Entregables

### E1: `cosmo_to_stellar.rs` — bridge S0→S1

```rust
/// Expandir cluster en estrellas.
///
/// - N_stars = kleiber_child_count(cluster.qe, Stellar)
/// - Masas: distribución de Salpeter emergente (power law de qe)
/// - Frecuencias: heredadas del cluster ± bandwidth
/// - Posiciones: dentro del radio del cluster, relajadas gravitacionalmente
pub fn expand_cluster(
    cluster: &EntitySlot,
    seed: u64,
    bandwidth: f64,
) -> Vec<StellarEntity>;
```

### E2: `stellar.rs` — simulación S1

```rust
/// Tick estelar: gravedad entre estrellas, nucleosíntesis como freq shift.
///
/// - Estrellas masivas (alta qe) emiten más (dissipation rate ↑)
/// - Frecuencia sube con edad (nucleosíntesis = enriquecimiento)
/// - Disco protoplanetario: gas orbitando estrella (angular momentum conservado)
pub fn stellar_tick(world: &mut SimWorldFlat, config: &StellarConfig);
```

**Nucleosíntesis emergente:**
No modelar reacciones nucleares. La frecuencia de una estrella sube con su edad
porque la dissipation concentra la energía remanente en modos más altos.
`freq_new = freq × (initial_qe / current_qe)^0.25` — a medida que pierde masa,
la frecuencia sube (blue shift = estrella más vieja/caliente).

### E3: `stellar_to_cosmo.rs` — agregación S1→S0

Al zoom-out: N estrellas → 1 cluster con qe = sum, freq = weighted mean.

## Tasks

- [ ] Crear `src/cosmic/bridges/cosmo_to_stellar.rs`
- [ ] Crear `src/cosmic/scales/stellar.rs`
- [ ] Distribución de masas estelares (power law emergente de Kleiber)
- [ ] Nucleosíntesis como freq shift por edad
- [ ] Discos protoplanetarios como entidades con angular momentum
- [ ] Tests:
  - `stellar_mass_distribution_power_law`
  - `nucleosynthesis_shifts_frequency_up`
  - `stellar_tick_conserves_qe`
  - `protoplanetary_disk_angular_momentum`
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Cluster con qe=10000 produce ~20-100 estrellas (Kleiber)
2. Distribución de masas sigue power law (exponente cerca de Salpeter ~2.35)
3. Estrellas viejas tienen freq más alta que jóvenes
4. `sum(stars.qe) < cluster.qe` (Axiom 4)
5. Al menos 1 estrella con disco protoplanetario si N_stars > 10
