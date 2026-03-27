# Track — Emergence Tiers (ET)

**Estado:** **ET-1–ET-16 implementados** (2026-03-25).

## Qué implementa

| Sprint | Módulo | Tier |
|--------|--------|------|
| ET-1 Associative Memory | `blueprint/equations/emergence/associations.rs` | T1-1 |
| ET-2 Theory of Mind | `blueprint/equations/emergence/other_model.rs` + `layers/other_model.rs` | T1-2 |
| ET-3 Cultural Transmission | `blueprint/equations/emergence/culture.rs` + `simulation/emergence/culture.rs` | T1-3 |
| ET-4 Infrastructure | `blueprint/equations/emergence/infrastructure.rs` + `simulation/emergence/infrastructure.rs` | T1-4 |
| ET-5 Obligate Symbiosis | `blueprint/equations/emergence/symbiosis.rs` + `layers/symbiosis.rs` | T2-1 |
| ET-6 Epigenetic Expression | `blueprint/equations/emergence/epigenetics.rs` + `layers/epigenetics.rs` | T2-2 |
| ET-7 Programmed Senescence | `blueprint/equations/emergence/senescence.rs` + `layers/senescence.rs` | T2-3 |
| ET-8 Dynamic Coalitions | `blueprint/equations/emergence/coalitions.rs` + `simulation/emergence/coalitions.rs` | T2-4 |
| ET-9 Multidimensional Niche | `blueprint/equations/emergence/niche.rs` + `layers/niche.rs` | T2-5 |
| ET-10 Multiple Timescales | `blueprint/equations/emergence/timescale.rs` + `layers/timescale.rs` | T3-1 |
| ET-11 Multiscale Info | `blueprint/equations/emergence/multiscale.rs` + `simulation/emergence/multiscale.rs` | T3-2 |
| ET-12 Continental Drift | `blueprint/equations/emergence/tectonics.rs` + `simulation/emergence/tectonics.rs` | T3-3 |
| ET-13 Geological Time LOD | `blueprint/equations/emergence/geological_lod.rs` + `simulation/emergence/geological_lod.rs` | T3-4 |
| ET-14 Institutions | `blueprint/equations/emergence/institutions.rs` + `simulation/emergence/institutions.rs` | T4-1 |
| ET-15 Language | `blueprint/equations/emergence/language.rs` + `layers/language.rs` | T4-2 |
| ET-16 Functional Consciousness | `blueprint/equations/emergence/self_model.rs` + `layers/self_model.rs` | T4-3 |

## Cobertura

- 16 BridgeKind markers registrados en `src/bridge/config.rs`
- 127 tests unitarios en `blueprint::equations::emergence::*`
- `src/simulation/emergence/` wired en `simulation/mod.rs`
- `cargo test --lib`: 1993 passed, 0 failed
