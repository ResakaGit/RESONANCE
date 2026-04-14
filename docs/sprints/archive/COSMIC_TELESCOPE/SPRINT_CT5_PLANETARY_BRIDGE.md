# CT-5: Planetary Bridge — Estrella → Planetas → Worldgen existente

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** CT-4
**ADR:** ADR-036 §D4 (S1→S2, S2→S3)

## Objetivo

Conectar la escala estelar (S1) con la planetaria (S2) y de ahí a la ecológica
(S3). Esta es la transición más natural porque S2→S3 **ya existe** (worldgen +
abiogenesis). Solo falta S1→S2 (estrella → planetas).

## Precondiciones

- CT-4 completado (escala estelar con estrellas + discos)
- Worldgen funcional (`worldgen/` completo)
- Abiogenesis funcional (`simulation/abiogenesis/`)

## Entregables

### E1: `stellar_to_planetary.rs` — bridge S1→S2

```rust
/// Expandir estrella con disco en sistema solar.
///
/// - N_planets ∝ disk.qe^Kleiber (3-12 típico)
/// - Órbitas: radios ∝ geometric series (Titius-Bode emergente)
/// - qe por planeta: decae con distancia a estrella (Axiom 7)
/// - Frecuencia: heredada de estrella ± bandwidth
/// - Zona habitable: donde T ∈ [liquid range] → water possible
pub fn expand_stellar_system(
    star: &EntitySlot,
    disk_qe: f64,
    seed: u64,
    bandwidth: f64,
) -> Vec<PlanetSpec>;

pub struct PlanetSpec {
    pub qe: f64,
    pub freq: f64,
    pub orbital_radius: f64,
    pub temperature: f64,       // derivada de distancia a estrella + qe
    pub matter_state: MatterState,
}
```

### E2: `planetary_to_ecological.rs` — bridge S2→S3 (wrapper)

```rust
/// Convertir PlanetSpec en EnergyFieldGrid + NucleusReservoir.
///
/// Este bridge es un wrapper delgado: el worldgen existente ya hace todo.
/// Solo traduce PlanetSpec → MapConfig equivalente.
pub fn planet_to_worldgen(spec: &PlanetSpec, seed: u64) -> MapConfig;
```

La estrella se convierte en el `EnergyNucleus` central del grid. Su frecuencia
es la frecuencia solar del campo. Los planetas son las condiciones iniciales
del `EnergyFieldGrid`.

### E3: Zona habitable emergente

No programar "zona habitable". Emerge naturalmente:
- Planeta muy cerca → temperatura alta → `MatterState::Plasma/Gas` → dissipation alta → vida improbable
- Planeta muy lejos → temperatura baja → `MatterState::Solid` → poca dinámica → vida improbable
- Planeta a distancia intermedia → `Liquid` → dissipation baja + dinámica → vida emerge

La coherencia de frecuencia con la estrella madre determina cuánta energía
absorbe el planeta (fotosíntesis emergente via Axiom 8 alignment).

## Tasks

- [ ] Crear `src/cosmic/bridges/stellar_to_planetary.rs`
- [ ] Crear `src/cosmic/bridges/planetary_to_ecological.rs`
- [ ] Distribución orbital (geometric series)
- [ ] Temperatura derivada de distancia + qe estrella (inverse square)
- [ ] Wrapper a MapConfig existente
- [ ] Tests:
  - `planetary_system_qe_conserved`
  - `orbital_radii_geometric`
  - `habitable_zone_is_liquid_state`
  - `planet_to_worldgen_produces_valid_mapconfig`
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Estrella con disco produce 3-12 planetas
2. Al menos 1 planeta en zona líquida si disk.qe suficiente
3. Worldgen existente funciona con MapConfig generado (abiogenesis activa)
4. `sum(planets.qe) < disk.qe` (Pool Invariant)
5. Planetas cercanos más calientes que lejanos (Axiom 7)
