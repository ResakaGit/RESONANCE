# Track: CIVILIZATION — Sociedad emergente de axiomas termodinámicos

Infraestructura + instituciones + lenguaje + agricultura = civilización.
Entidades construyen estructuras permanentes, establecen reglas de acceso,
comercian recursos, y acumulan conocimiento generacional.

**Invariante:** Ninguna ley social se programa. Emerge de cooperación + lenguaje + tools.

---

## Qué ya existe

| Componente | Uso |
|-----------|-----|
| `infrastructure_update_system` (ET-4) | Entities construyen bonus shared |
| `infrastructure_intake_bonus_system` (ET-4) | Beneficio grupal de infraestructura |
| `institutions` stub (ET-14) | Placeholder para reglas emergentes |
| `coalition_stability_system` (ET-8) | Grupos estables con intake bonus |
| `LanguageCapacity` | Comunicación simbólica |
| `cooperation_evaluation_system` (AC-5) | Nash alliance detection |
| Farming (TU-3) | Producción de recursos estable |

## Sprints (4)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [CV-1](SPRINT_CV1_PERSISTENT_STRUCTURES.md) | Persistent Structures | 1 sem | TU ✅ | Entidades "edificio": alta bond, no senescence, shared resource |
| [CV-2](SPRINT_CV2_ACCESS_RULES.md) | Access Rules | 1 sem | CV-1, EL ✅ | `access_rule()` — quién puede usar qué (coalition-based) |
| [CV-3](SPRINT_CV3_TRADE.md) | Resource Trade | 1 sem | CV-2 | `trade_proposal()` — intercambio de qe mediado por lenguaje |
| [CV-4](SPRINT_CV4_BATCH_WIRING.md) | Batch Integration | 1 sem | CV-3 | Civilization en batch + civilization_score observable |

## Arquitectura

```
src/blueprint/equations/
├── civilization.rs              ← CV-1/2/3: structure_durability, access_rule, trade_value
src/blueprint/constants/
├── civilization.rs              ← Constantes derivadas
src/batch/systems/
├── civilization.rs              ← CV-4: batch systems
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 2 | Estructura = pool de energía compartido. Σ access ≤ capacity. |
| 4 | Construir y mantener estructuras cuesta energía. |
| 5 | Trade conserva: Σ qe antes = Σ qe después (no se crea riqueza de la nada). |
| 6 | Reglas sociales emergen de qué coaliciones sobreviven. |
| 7 | Acceso limitado por distancia a la estructura. |
| 8 | Trade agreements modulados por frequency alignment (confianza ∝ similitud). |

## Constantes derivadas

| Constante | Derivación |
|-----------|-----------|
| `STRUCTURE_BUILD_COST` | `DISSIPATION_SOLID × 1000` — inversión alta |
| `STRUCTURE_MAINTENANCE` | `DISSIPATION_SOLID × 5` — costo por tick |
| `TRADE_FRICTION` | `DISSIPATION_SOLID × 10` — costo por transacción |
| `ACCESS_RANGE` | `DENSITY_SCALE` — radio de acceso = tamaño del grid cell |

## Esfuerzo total: ~4 semanas, ~400 LOC, ~40 tests
