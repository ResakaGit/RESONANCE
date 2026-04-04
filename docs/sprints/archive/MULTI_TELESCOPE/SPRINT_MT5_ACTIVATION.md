# MT-5: Activación y Métricas Multi-Nivel

**Objetivo:** Extender TelescopeSummary para reportar métricas de todos los niveles del stack. Implementar coherence_length dinámico (derivado de régimen). Conectar visibilidad por nivel al dashboard. Este sprint no crea math nueva — conecta piezas existentes.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo (wiring, no lógica nueva)
**Bloqueado por:** MT-4 (pipeline del stack)
**Desbloquea:** Casos de uso CU-1 a CU-5 del README

---

## Entregables

### 1. Coherence Length Dinámico

En `src/batch/telescope/activation.rs` (extender archivo existente):

```rust
/// Calcula longitud de coherencia desde métricas de régimen.
/// Estasis: longitud grande (se puede ver lejos). Transición: longitud chica.
///
/// Derivado de: H (persistencia), ρ₁ (inercia), λ_max (estabilidad).
/// Fórmula: base × H × (1 + |λ_max|⁻¹) × (1 - ρ₁²)
pub fn dynamic_coherence_length(
    metrics: &RegimeMetrics,
    base_coherence: f32,
) -> f32
```

### 2. TelescopeSummary Multi-Nivel

Extender `TelescopeSummary` existente o crear `StackSummary`:

```rust
/// Resumen del stack multi-nivel para dashboard.
pub struct StackSummary {
    pub active_levels: u8,
    pub total_reach: u64,
    pub coherence_length: f32,
    pub levels: [LevelSummary; MAX_LEVELS],
}

/// Resumen de un nivel individual.
#[derive(Clone, Copy, Debug, Default)]
pub struct LevelSummary {
    pub k: u32,
    pub visibility: f32,           // V de Englert
    pub last_diff_class: u8,       // 0=Perfect, 1=Local, 2=Systemic
    pub projection_accuracy: f32,  // media de últimas reconciliaciones de este nivel
}

/// Genera StackSummary desde el estado actual.
pub fn stack_summary(
    stack: &TelescopeStack,
    history: &ReconciliationHistory,
) -> StackSummary
```

### 3. LOD Dinámico desde Stack

El `lod_level_from_k` existente opera sobre un solo K. Para multi-nivel, el LOD refleja el nivel MÁS GRUESO confiable:

```rust
/// LOD dinámico desde stack: usa el nivel más alto con V < 0.5 (más colapsado que onda).
pub fn lod_from_stack(stack: &TelescopeStack) -> u32
```

---

## Preguntas para tests

### Coherence Length
1. Estasis (H=0.8, ρ₁=0.3, λ=-0.05): ¿coherence_length > DEFAULT? (estable = ve lejos)
2. Transición (H=0.4, ρ₁=0.9, λ=-0.001): ¿coherence_length < DEFAULT? (inestable = ve cerca)
3. coherence_length siempre > 0 (nunca negativo o cero)
4. coherence_length es finito para cualquier input finito

### StackSummary
5. Stack con 1 nivel: ¿summary.active_levels = 1, total_reach = K₀?
6. Stack con 3 niveles: ¿summary muestra 3 LevelSummary con visibility creciente?
7. LevelSummary.projection_accuracy = fracción de PERFECTs en historial de ese nivel

### LOD desde Stack
8. Stack con todos niveles V < 0.5: ¿LOD = lod_level_from_k(K del nivel más alto)?
9. Stack con niveles altos V > 0.5: ¿LOD excluye niveles "onda pura" (no confiables)?
10. Stack con 1 nivel: ¿comportamiento idéntico a lod_level_from_k existente?

### Compatibilidad
11. Todos los tests existentes de activation.rs siguen verdes
12. TelescopeSummary original sigue funcionando para active_levels=1

---

## Integración

- **Consume:** MT-3 (TelescopeStack), MT-4 (StackTickResult), MT-1 (speculative_visibility)
- **Modifica:** `activation.rs` (agrega funciones), `constants/temporal_telescope.rs` (si se necesitan constantes para coherence_length)
- **No modifica:** Nada del core batch, ningún sistema, ningún axioma
