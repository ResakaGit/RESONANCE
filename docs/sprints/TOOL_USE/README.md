# Track: TOOL_USE — Herramientas y agricultura emergentes

Entidades que modifican otras sin consumirlas (herramientas) y que mantienen
vivas a otras para cosechar periódicamente (agricultura).

**Invariante:** Nadie programa "usa piedra". Emerge de: intent=modify + injector + target.

---

## Qué ya existe

| Componente | Uso |
|-----------|-----|
| `AlchemicalInjector` (L8) | Modificar propiedades de otra entidad |
| `WillActuator` (L7) | Intención dirigida |
| `SymbiosisLink` | Relación mutualism/parasitism (farming = mutualism dirigido) |
| `TrophicConsumer` | Herbivore/carnivore (farming = "herbivore que no mata") |
| `cooperation_evaluation_system` | Nash alliance (farmer-crop = alliance asimétrica) |

## Sprints (4)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [TU-1](SPRINT_TU1_MODIFY_INTENT.md) | Modify Intent | 1 sem | EI ✅ | BehaviorMode::Modify — "cambiar sin consumir" |
| [TU-2](SPRINT_TU2_TOOL_CRAFTING.md) | Tool Crafting | 1 sem | TU-1 | Injector modifica bond_energy de target → "tool" |
| [TU-3](SPRINT_TU3_FARMING.md) | Farming | 1 sem | TU-1 | SymbiosisLink mode: "harvest without killing" |
| [TU-4](SPRINT_TU4_BATCH_WIRING.md) | Batch Integration | 1 sem | TU-2, TU-3 | Tool use + farming en batch + tool_use_rate observable |

## Arquitectura

```
src/blueprint/equations/
├── tool_use.rs                  ← TU-1/2: modify_intent_score, craft_tool, tool_effectiveness
├── farming.rs                   ← TU-3: harvest_yield, farming_efficiency, crop_maintenance
src/blueprint/constants/
├── tool_use.rs                  ← Constantes derivadas
src/batch/systems/
├── tool_use.rs                  ← TU-4: batch systems
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 4 | Crafting cuesta energía. Farming requiere maintenance cost. |
| 6 | Tool use emerge: organismos que craftan tienen acceso a más recursos. |
| 7 | Tools solo funcionan por contacto (distancia cero). |

## Esfuerzo total: ~4 semanas, ~350 LOC, ~40 tests
