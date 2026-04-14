# Sprint AP-5: Persistence Property Test — Operacionalización del cap. 10

**ADR:** —
**Esfuerzo:** 1 semana
**Bloqueado por:** AP-4
**Desbloquea:** AP-6

## Contexto

El núcleo destilado del paper (cap. 10):

> Lo que persiste es aquello que encontró una forma de copiarse antes de disiparse.

Esto es un invariante falsable. Si lo escribimos como property test que pasa con sopas aleatorias, hemos operacionalizado el axioma.

## Principio

```
proptest:
  Para cualquier seed s ∈ [0, 1000):
    network = random_reaction_network(s, n_species=16, n_reactions=32, p_catalysis=0.3)
    food    = random_food_set(s, k=4)
    grid    = uniform_initial_concentration(food, 100.0)

    simulate(grid, network, food, ticks=10_000)

    closures_t0    = closures detected at tick 100  (after equilibration)
    closures_final = closures detected at tick 10000

    Para cada c ∈ closures_t0:
      if c.id_hash ∈ closures_final.ids:
        // sobrevivió → debe haberse replicado O sostenido
        assert(
          fission_events.count(c.lineage) ≥ 1
          OR
          mean(c.k_stability over [9000..10000]) ≥ 1.0
        )
```

Si esto pasa para 1000 sopas aleatorias, el simulador demuestra el invariante.

## Entregable

1. `random_reaction_network(seed, n_species, n_reactions, p_catalysis) → ReactionNetwork` — generator
2. `random_food_set(seed, k) → Vec<SpeciesId>` — generator
3. `proptest_persistence` en `tests/property_autopoiesis.rs`
4. `harness::run_soup(network, food, ticks) → SoupReport` — pure orchestrator (no Bevy)
5. `SoupReport { closures_initial, closures_final, fissions, k_stability_history }`
6. CI gate: AP-5 corre en cada PR con 100 cases (full 1000 en nightly)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | Random network generator | `src/use_cases/experiments/autopoiesis/random_soup.rs` | 4 |
| 2 | `harness::run_soup` headless | `src/use_cases/experiments/autopoiesis/harness.rs` | 4 |
| 3 | `SoupReport` struct + serde | `src/use_cases/experiments/autopoiesis/report.rs` | 2 |
| 4 | Property test ≥1000 cases | `tests/property_autopoiesis.rs` | 1 (proptest) |
| 5 | CI workflow `autopoiesis-nightly.yml` | `.github/workflows/autopoiesis-nightly.yml` | — |

## Criterios de aceptación

- [ ] Property test pasa con 1000 cases sin shrinks falsos
- [ ] Tiempo total: < 60s con 100 cases (PR), < 600s con 1000 (nightly)
- [ ] Sopa trivial (sin catálisis cruzada) → 0 closures sobrevivientes (control)
- [ ] Sopa con RAF embebida + food abundante → ≥1 closure sobrevive Y se replica ≥1 vez
- [ ] Sopa con RAF + food agotable → closure muere antes de 10k ticks (extinción documentada)
- [ ] Reporte JSON válido para tracking inter-build

## Significado

**Si este test pasa**, el simulador **demuestra** —no asume— el invariante destilado del paper. Es el cierre del arco. Antes de AP-5: un simulador termodinámico capaz. Después: una máquina que disipa copiándose.
