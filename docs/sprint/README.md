# Sprint: Fauna Simulation & Ecological Completeness

## Objetivo

Completar los sistemas faltantes para que Resonance pueda simular **fauna real** con comportamiento emergente, cadenas tróficas, especiación, y homeostasis — todo sobre la arquitectura existente de 14 capas ortogonales.

## Estado Actual (Post-Audit)

| Métrica | Valor |
|---------|-------|
| Systems ACTIVE | 44 (incl. `tension_field_system`, `homeostasis_system`) |
| Systems PARTIAL | 12 |
| Components huérfanos | 0 (verificación cruzada confirmó todos activos) |
| Components PLANNED (sin system aún) | 1 (`PerformanceCachePolicy` — V5) |
| Equations implementadas | 150+ (29 archivos, 0 huérfanas) |
| Constants | 500+ (48 archivos) |
| Tests existentes | ~920+ |

**Nota post-verificación**: Los componentes TensionField (L11), Homeostasis (L12), AlchemicalForge (L5) y VisionBlocker tienen systems activos o son markers ECS. El audit inicial reportó falso positivo por no detectar queries con filtros `With<T>`/`Without<T>` ni accesos por método.
- `tension_field_system` → ACTIVO en `structural_runtime.rs:98`, Phase::AtomicLayer
- `homeostasis_system` → ACTIVO en `structural_runtime.rs:184`, Phase::ChemicalLayer (adapta freq hacia host pressure, emite HomeostasisAdaptEvent)
- `AlchemicalForge` → data component con métodos, accedido por lógica de grimoire/abilities
- `VisionBlocker` → marker de filtro en fog_of_war.rs (`Without<VisionBlocker>`)

## Sistemas Nuevos: 32 sistemas en 9 dominios

```
DOMINIO                         SYSTEMS  PRIORIDAD  DEPENDENCIAS
─────────────────────────────────────────────────────────────────
D1  Behavioral Intelligence     5        P0         L7, L5, SpatialIndex
D2  Trophic & Predation         4        P0         D1, L0, L5, ecology eqs
D3  Locomotion Energy Cost      3        P1         L3, L5, L6, topology
D4  Homeostasis & Thermo        2+1exist P1         L12, L4, morphogenesis eqs
D5  Sensory & Perception        3        P2         L2, SpatialIndex, EcoField
D6  Social & Communication      3        P2         D1, D5, L13
D7  Reproductive Isolation      3        P1         D2, population eqs
D8  Morphological Adaptation    3        P2         D4, organ_inference, lifecycle
D9  Ecological Dynamics          3        P2         D2, D7, NutrientField
─────────────────────────────────────────────────────────────────
REPARACIONES PARCIALES          2        P0         Varios
TOTAL                           32
```

## Orden de Ejecución (DAG de dependencias)

```
Fase 0 (Cimientos):
  ├─ REPAIR: Completar 12 systems parciales
  ├─ D1: Behavioral Intelligence ← fundamento para toda fauna
  └─ D2: Trophic & Predation ← cadena alimentaria básica

Fase 1 (Metabolismo):
  ├─ D3: Locomotion Energy Cost ← movimiento tiene precio
  ├─ D4: Homeostasis & Thermo ← termorregulación
  └─ D7: Reproductive Isolation ← especiación

Fase 2 (Ecología):
  ├─ D5: Sensory & Perception ← conciencia del entorno
  ├─ D6: Social & Communication ← manada, jerarquía
  ├─ D8: Morphological Adaptation ← forma sigue función
  └─ D9: Ecological Dynamics ← censo, capacidad de carga, sucesión
```

## Archivos del Sprint

```
docs/sprint/
├── README.md                          ← este archivo
├── rules/
│   ├── anti_patterns.md               ← qué NO hacer (zero tolerance)
│   ├── design_rules.md                ← reglas de diseño por system
│   └── verification_checklist.md      ← checklist pre-merge
├── patterns/
│   ├── orchestrator.md                ← patrones del orquestador
│   ├── performance.md                 ← patrones de rendimiento
│   └── coupling.md                    ← reducción de acoplamiento
└── domains/
    ├── d0_repairs.md                  ← completar systems parciales
    ├── d1_behavioral_intelligence.md  ← FSM/Utility AI
    ├── d2_trophic_predation.md        ← cadena trófica
    ├── d3_locomotion_cost.md          ← energía de movimiento
    ├── d4_homeostasis_thermo.md       ← termorregulación
    ├── d5_sensory_perception.md       ← percepción
    ├── d6_social_communication.md     ← comportamiento social
    ├── d7_reproductive_isolation.md   ← especiación
    ├── d8_morphological_adaptation.md ← adaptación morfológica
    └── d9_ecological_dynamics.md      ← dinámica poblacional
```

## Principios Rectores

1. **Composición, no herencia**: Cada system nuevo lee/escribe capas existentes. Cero componentes nuevos salvo los estrictamente necesarios.
2. **Emergencia sobre prescripción**: El comportamiento del lobo NO se programa — emerge de hambre (L0 baja) + sensory (detectar presa) + will (perseguir) + trophic (consumir).
3. **Termodinámica primero**: Todo tiene costo energético. Moverse cuesta, pensar cuesta, termorregular cuesta. Sin energía gratis.
4. **Determinismo reproducible**: Mismo estado inicial → mismo resultado. Sin RNG no-determinista en systems.
5. **Throttling por diseño**: Todo system que itere N² usa cursors + budgets por frame.
