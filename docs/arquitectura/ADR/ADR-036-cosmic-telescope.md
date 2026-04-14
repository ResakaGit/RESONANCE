# ADR-036: Cosmic Telescope — Simulación multi-escala con colapso observacional

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Decisores:** Resonance Core
**Contexto:** ADR-015 (Temporal Telescope), ADR-016 (Multi-Telescope Stack)
**Extiende:** ADR-016

## Contexto

ADR-015/016 resuelven la exploración **temporal** — proyectar el futuro de un
mundo a escala fija. No existe exploración **espacial** a múltiples escalas:
Big Bang → clusters → estrellas → planetas → ecosistemas → moléculas.

La física real opera igual: el estado de una región no observada es una
superposición de microestados compatibles con los observables macroscópicos
(energía total, temperatura, frecuencia dominante). Al observar (zoom), el
estado colapsa a UNA realidad concreta determinada por el seed.

### Infraestructura existente

| Componente | Archivo | Qué aporta |
|------------|---------|------------|
| TelescopeStack | `batch/telescope/stack.rs` | 8 niveles temporales, collapse + re-emanation |
| SimWorldFlat | `batch/arena.rs` | ~100KB arena memcpy-friendly, 1024 entities |
| LOD | `worldgen/lod.rs` | Near/Mid/Far bands por distancia |
| TensionField (L11) | `layers/tension_field.rs` | Fuerza gravitacional InverseSquare |
| Abiogenesis | `simulation/abiogenesis/mod.rs` | coherence > dissipation → vida |
| Component groups | `entities/component_groups.rs` | Factories de spawning por tuple |
| Go model + REMD | `blueprint/equations/go_model.rs` | Folding proteico con Axiom 8 |
| personal_universe | `bin/personal_universe.rs` | Seed determinista → universo único |
| EnergyFieldGrid | `worldgen/field_grid.rs` | Grid 2D de energía + frecuencia |
| NucleusReservoir | `worldgen/nucleus.rs` | Emisión finita, ciclo cerrado |

### Problema

No hay forma de simular el Big Bang y hacer zoom hasta ver proteínas plegándose
dentro de un organismo en un planeta de un sistema estelar de un cluster galáctico.
Cada escala existe como binario independiente sin puente entre escalas.

## Decision

### D1: Jerarquía de escalas espaciales (ScaleLevel)

5 niveles, cada uno con su propia SimWorldFlat y dt apropiado:

```
S0  Cosmológico   N clusters, gravitación (L11), dt_cosmo
S1  Estelar       Estrellas + gas + protoplanetas, dt_stellar
S2  Planetario    Superficie, EnergyFieldGrid, dt_planet (= dt actual)
S3  Ecológico     Abiogenesis, organismos, evolución, dt_eco (= tick actual)
S4  Molecular     particle_lab / fold_go, dt_mol = 0.005
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleLevel { Cosmological, Stellar, Planetary, Ecological, Molecular }
```

Cada nivel tiene su propio `SimWorldFlat` (o `MdWorld` para S4). Solo el nivel
observado corre a resolución completa. Los demás corren coarsened (D3).

### D2: Zoom = colapso observacional

Al hacer zoom en una entidad del nivel S_n, se:

1. **Congela** la entidad padre en S_n
2. **Infiere** el estado interno respetando axiomas (D2.1–D2.4)
3. **Instancia** S_{n+1} como nuevo SimWorldFlat
4. **Simula** S_{n+1} a resolución completa

Al hacer zoom-out, se **agrega** el estado de S_{n+1} de vuelta al padre.

**D2.1 — Pool Invariant (Axiom 2):**
```
sum(child_i.qe) <= parent.qe
qe_available = parent.qe × (1 - DISSIPATION[parent.state])    ← Axiom 4
```

**D2.2 — Distribución de energía:**
```
N_children = kleiber_count(parent.qe)       ← Kleiber scaling
qe_i ~ Dirichlet(alpha=1) × qe_available   ← uniforme, luego relajación
```

**D2.3 — Frecuencias (Axiom 8):**
```
freq_i ~ Normal(parent.freq, COHERENCE_BANDWIDTH)
```
Los hijos heredan el espectro del padre con dispersión acotada por bandwidth.

**D2.4 — Posiciones:**
```
pos_i ~ uniform dentro del radio del padre
```
Luego relajación via fuerzas (Coulomb/LJ/Go según escala) durante N_relax steps.

**D2.5 — Seed determinista:**
```
zoom_seed = hash(parent_entity_id, scale_level, user_seed)
```
Mismo padre + mismo user_seed = misma realidad. Distinto user_seed = universo
paralelo válido. Esto es el **multiverso**: cada zoom es un branch.

### D3: Background coarsening

Niveles no observados siguen corriendo pero a tasa reducida:

```
tick_rate(level) = match distance_from_observed {
    0 => 1,              // resolución completa
    1 => K_PARENT,       // 1 tick coarse = K ticks finos
    2 => K_PARENT²,      // aún más grueso
    _ => FROZEN,          // snapshot estático, no corre
}
```

El nivel cosmológico (S0) siempre corre — es el contexto global. Solo se congela
si el observador está en S4 (molecular), a 4 escalas de distancia.

**Coarsening conserva axiomas:**
- Axiom 2: `sum(qe)` se preserva en la agregación
- Axiom 4: `dissipation` se aplica proporcional a ticks saltados
- Axiom 5: `total_qe` monotónicamente decrece (nunca crece por coarsening)

### D4: Puentes entre escalas

Cada transición S_n → S_{n+1} tiene un **bridge** específico:

| Transición | Bridge | Qué infiere |
|------------|--------|-------------|
| S0 → S1 | `cosmo_to_stellar` | Cluster → N estrellas. Masas ∝ qe^Kleiber |
| S1 → S2 | `stellar_to_planetary` | Estrella → sistema solar. Planetas por Titius-Bode emergente |
| S2 → S3 | `planetary_to_ecological` | Planeta → EnergyFieldGrid + nuclei. **Ya existe** (worldgen) |
| S3 → S4 | `ecological_to_molecular` | Organismo → proteínas. Frecuencias de residuos ∝ freq del organismo |

| Transición | Bridge | Qué agrega |
|------------|--------|------------|
| S1 → S0 | `stellar_to_cosmo` | N estrellas → 1 cluster. qe = sum, freq = dominante |
| S2 → S1 | `planetary_to_stellar` | Sistema → 1 estrella. qe = sum + radiación |
| S3 → S2 | `ecological_to_planetary` | Grid → 1 planeta. qe = total_field, T = mean |
| S4 → S3 | `molecular_to_ecological` | Proteínas → 1 organismo. Estado = folding Q |

### D5: Integración con Temporal Telescope

Cada ScaleLevel tiene su propio TelescopeStack (ADR-016). El telescopio temporal
opera **dentro** de cada escala espacial:

```
CosmicTelescope
├── S0: TelescopeStack (K=1024, reach ~10^9 ticks cósmicos)
├── S1: TelescopeStack (K=256, reach ~10^7 ticks estelares)
├── S2: TelescopeStack (K=64, estaciones/ciclos geológicos)
├── S3: TelescopeStack (K=16, generaciones de vida)
└── S4: TelescopeStack (K=4, pasos MD cortos)
```

### D6: Visualización

Cámara con transición suave entre escalas. Al hacer zoom:

1. Cámara se acerca al cluster seleccionado
2. Fade out del nivel actual
3. Spawn del nivel inferior con partículas expandiéndose
4. Fade in del nuevo nivel
5. HUD muestra: escala actual, qe total, edad, seed, regime

Al máximo zoom-out: vista del universo completo con clusters como puntos.
Al máximo zoom-in: átomos individuales de una proteína.

### D7: Multiverso

Cada zoom fork-ea la realidad. El usuario puede:

1. Zoom en cluster A con seed 1 → Realidad A1
2. Zoom-out, zoom en cluster A con seed 2 → Realidad A2
3. Comparar A1 vs A2

Un `MultiverseLog` registra todos los branches visitados. Las propiedades
estadísticas (¿con qué frecuencia emerge vida?) son un observable del sistema.

Esto es análogo a la interpretación de muchos mundos: cada observación (zoom)
elige un eigenstate del cluster. El seed es la fase del observador.

## Consecuencias

### Positivas

- **Simulación Big Bang → vida** en un solo binario
- **Multiverso emergente** de la arquitectura, no programado
- **Reutiliza 100%** de la infraestructura existente (telescope, worldgen, abiogenesis, MD)
- **Axiomáticamente cerrado** — todo derivado de los 4 fundamentales
- **Demo inigualable** para el paper de Zenodo
- **Escalable**: solo el nivel observado usa CPU completa

### Negativas

- **Complejidad de state management** entre 5 niveles simultáneos
- **Transiciones de escala** requieren tuning visual
- **Inferencia S0→S1 y S1→S2** son nuevas — necesitan validación

### Riesgos

| Riesgo | Mitigación |
|--------|------------|
| Inconsistencia al zoom-out después de simular zoom-in | Agregación conservativa: qe del padre = sum(hijos) × (1 - dissipation) |
| Performance con múltiples SimWorldFlat | Solo 2 activos (observado + 1 background). Máximo ~200KB RAM |
| Inferencia genera estados no físicos | Relajación post-spawn (N steps con fuerzas). Estados no físicos se corrigen solos |

## Alternativas consideradas

| Alternativa | Rechazada porque |
|-------------|------------------|
| Simulación todo-a-la-vez (N-body cosmológico + MD simultáneo) | Imposible computacionalmente. 10^80 partículas no caben |
| Pre-computar todos los niveles | Viola el colapso observacional. El estado debe ser indeterminado hasta la observación |
| Solo 3 niveles (cosmo, eco, mol) | Pierde la transición estelar/planetaria. La narrativa necesita el viaje visual completo |
| GPU para todas las escalas | Viola regla de no-unsafe y no-crates. std::thread::scope es suficiente |

## Referencias

- ADR-015: Temporal Telescope — dual-timeline speculative execution
- ADR-016: Multi-Telescope Stack — 8-level quantum-inspired hierarchy
- `batch/telescope/stack.rs` — TelescopeStack implementation
- `batch/arena.rs` — SimWorldFlat definition
- `worldgen/lod.rs` — LOD bands
- `simulation/abiogenesis/mod.rs` — axiomatic emergence
- `blueprint/equations/go_model.rs` — protein folding
- Englert 1996, "Fringe Visibility and Which-Way Information"
- Kauffman 1993, "The Origins of Order" — emergence at phase transitions
