# Track: MD_REFACTOR — De prototipo funcional a motor competitivo

## Filosofía

> "No redescubramos. Aprendamos y mejoremos." — Filosofía AstraZeneca

GROMACS tiene 30 años de optimización. LAMMPS tiene 20. Pero también tienen 30 años
de deuda técnica, APIs de Fortran envueltas en C++, y decisiones de 1995 que no pueden
revertir sin romper 10,000 scripts de usuario.

**Nosotros no tenemos esa deuda.** Tenemos un motor limpio, axiomático, en Rust con
ownership y zero-cost abstractions. La ventaja no es en features — es en velocidad
de iteración y corrección por diseño.

**Estrategia:** tomar lo mejor de la industria MD (algoritmos probados), descartando
su deuda (APIs, formatos legacy, compromises históricos).

---

## Auditoría: 26 issues encontrados

| Severidad | Cantidad | Categoría principal |
|-----------|----------|---------------------|
| CRITICAL | 2 | Data corruption en linked lists (neighbor_list, spatial_tree) |
| HIGH | 5 | Física incorrecta (force clipping), duplicación (erfc), magic numbers |
| MEDIUM | 10 | Performance (O(N²), mask rebuilt), numerical grad, f32/f64 mixing |
| LOW | 9 | Documentación, unused params, edge cases |

---

## Sprints

### R0: Critical Bug Fixes (1 día)

**Prioridad:** BLOQUEANTE. No hacer nada más hasta que estos estén resueltos.

| Issue | Archivo | Fix |
|-------|---------|-----|
| Linked list corruption | `neighbor_list.rs:176` | Eliminar líneas duplicadas |
| Linked list corruption | `spatial_tree.rs:176` | Eliminar líneas duplicadas |
| LJ force clipping | `md_observables.rs:193-204` | Eliminar clamp, softening correcto |
| Lost particles in tree | `spatial_tree.rs:222` | Link explícito para celdas degeneradas |

### R1: Consolidar constantes y eliminar duplicación (3 días)

**ADR:** [ADR-026 Shared Math Utilities](../../arquitectura/ADR/ADR-026-shared-math-utilities.md)

| Acción | Detalle |
|--------|---------|
| Crear `equations/special_functions.rs` | `erfc_approx()`, `erf_approx()` — una sola implementación |
| Crear `equations/unit_conversion.rs` | `DEG_TO_RAD`, `RAD_TO_DEG`, `KCAL_TO_KJ`, `ANGSTROM_TO_NM` |
| Eliminar TIP3P hardcoded en constraints.rs | Importar de `batch::ff::water` |
| Nombrar todos los magic numbers | `NUMERICAL_GRAD_STEP`, `BARNES_HUT_THETA`, `BRUTE_FORCE_THRESHOLD` |
| Documentar coeficientes erf | Referencia Abramowitz & Stegun 7.1.26 |

### R2: Performance — de O(N²) a O(N log N) (2 semanas)

**ADRs:** [ADR-027 PME vs Bare Ewald](../../arquitectura/ADR/ADR-027-pme-vs-bare-ewald.md),
[ADR-028 Analytical vs Numerical Gradients](../../arquitectura/ADR/ADR-028-analytical-gradients.md)

| Acción | Impacto | Referencia industria |
|--------|---------|---------------------|
| PME (Particle Mesh Ewald) | Ewald O(N·K³) → O(N log N) | GROMACS: PME es el default desde 2001 |
| Analytical angle/dihedral gradients | 6x speedup en bonded forces | AMBER: analytical desde v4 (1995) |
| Cache native mask en REMD | Eliminar N² alloc por step | Cualquier Go model implementation |
| Cell list para repulsión no-nativa | O(N²) → O(N) para Go model | LAMMPS: cell list universal |
| Verlet neighbor list con skin | Rebuild cada ~20 steps, no cada step | Allen & Tildesley, Ch. 5 |

### R3: Precisión y consistencia de tipos (1 semana)

**ADR:** [ADR-029 f64-Everywhere Migration](../../arquitectura/ADR/ADR-029-f64-everywhere.md)

| Acción | Detalle |
|--------|---------|
| Migrar bonded.rs a f64 | Actualmente f32 params en f64 world → truncation |
| Unificar position_step signatures | f32 2D solo para legacy batch, f64 3D para MD |
| Eliminar casts innecesarios | `as f64` / `as f32` donde ya es el tipo correcto |
| kinetic_temperature parametrizar dim | 2D vs 3D, no hardcoded |

### R4: Robustez y edge cases (3 días)

| Acción | Detalle |
|--------|---------|
| SHAKE non-convergence detection | Log warning, no panic silencioso |
| Bounds check en neighbor list | Assert particles in box antes de cell assignment |
| Remove _d_target de rattle_pair | Parámetro unused |
| Guard aa_type en go_model | Warn para tipos desconocidos |
| Document Barnes-Hut trade-offs | Axiom 5 approximate para tree |

### R5: Estrategias de industria — algoritmos avanzados (4 semanas)

**ADRs:** [ADR-030 SETTLE vs SHAKE](../../arquitectura/ADR/ADR-030-settle-vs-shake.md),
[ADR-031 Multiple Timestepping](../../arquitectura/ADR/ADR-031-multiple-timestepping.md),
[ADR-032 Implicit Solvent Option](../../arquitectura/ADR/ADR-032-implicit-solvent.md)

**De GROMACS aprendemos:**
- SETTLE > SHAKE para agua (algebraic, no iterativo, 3x más rápido)
- LINCS > SHAKE para bonds generales (estable, parallelizable)
- PME con FFT sobre grid 3D (requiere crate `rustfft` — ADR-027)
- Virtual sites eliminan DOF y permiten dt más grande

**De LAMMPS aprendemos:**
- Domain decomposition para MPI (nosotros: rayon tasks)
- Hybrid pair styles (combinar Go + LJ + Coulomb en un loop)
- Compute/fix architecture (nuestro equivalente: systems pipeline)

**De OpenMM aprendemos:**
- Custom force expressions (nosotros: Axiom 8 modulation es exactamente esto)
- Langevin middle integrator (más estable que velocity Verlet + Langevin separado)

**De NAMD aprendemos:**
- Multiple timestepping (r-RESPA): bonded cada dt, non-bonded cada 2dt, long-range cada 4dt
- Esto es 4x speedup sin pérdida de accuracy

**De Desmond (Schrödinger) aprendemos:**
- u-series para long-range (alternativa a PME, más cache-friendly)
- Gaussian split Ewald (mejor que standard PME para GPU)

**De la industria farmacéutica (AstraZeneca/Novartis) aprendemos:**
- Enhanced sampling: metadynamics, umbrella sampling, steered MD
- Free energy perturbation (FEP) para ΔΔG de mutaciones
- Nuestro Axiom 8 frequency shift ES un FEP natural — mutation = Δf

---

## Estrategias por sector

### Estrategia A: Pharma (AstraZeneca model)
**Meta:** Calcular ΔΔG de mutaciones vía frequency perturbation.
**Prioridad:** R-RESPA + PME + FEP adapter sobre Axiom 8.
**ROI:** Si frequency shift predice estabilidad → herramienta de drug design.

### Estrategia B: Materials Science (LAMMPS model)
**Meta:** Simular materiales con interacciones customizadas.
**Prioridad:** Hybrid pair styles + domain decomposition + custom potentials.
**ROI:** Axiom 8 como "programmable potential" — nuevo paradigma de materiales con interacciones oscilatorias.

### Estrategia C: Structural Biology (GROMACS model)
**Meta:** Folding y docking de proteínas realistas.
**Prioridad:** PME + enhanced sampling (metadynamics) + all-atom FF.
**ROI:** Si Go + Axiom 8 outperforms classical Go → publicación de alto impacto.

### Estrategia D: Bioinformatics/ML (AlphaFold-adjacent)
**Meta:** Training data para modelos de ML de folding.
**Prioridad:** Batch REMD paralelo (ya tenemos rayon) + trajectory output.
**ROI:** Datasets de folding con frequency coherence como feature novel.

### Estrategia recomendada: **C primero, A después.**
Structural biology valida la ciencia. Pharma la monetiza.

---

## Orden de ejecución

```
R0 (1 día)     ──→ R1 (3 días) ──→ R3 (1 semana)
                                        │
                    R4 (3 días)  ───────┤
                                        ↓
                                   R2 (2 semanas)
                                        │
                                        ↓
                                   R5 (4 semanas) ← requires ADR decisions
```

**Total:** ~8 semanas. Parallelizable: R1+R4 simultáneo.

## Axiom Compliance Matrix

| Refactoring | Axiom 1 | Axiom 2 | Axiom 4 | Axiom 7 | Axiom 8 |
|-------------|---------|---------|---------|---------|---------|
| PME | — | — | — | ✓ exact long-range | — |
| SETTLE | — | ✓ no work | — | — | — |
| r-RESPA | ✓ energy split | ✓ conserved | ✓ dissip per scale | ✓ per-range dt | — |
| Analytical grad | ✓ exact energy | — | — | — | — |
| FEP via Axiom 8 | ✓ Δqe | — | ✓ ΔΔG | — | ✓ Δf = mutation |
| Metadynamics | ✓ bias potential | — | ✓ enhanced dissip | — | ✓ CV = coherence |

---

## Métricas de éxito

| Métrica | Actual | Target R2 | Target R5 | GROMACS ref |
|---------|--------|-----------|-----------|-------------|
| N=1K LJ steps/sec | ~100 | ~1,000 | ~5,000 | ~50,000 |
| Ewald N=1K | O(N·K³) | O(N log N) PME | O(N log N) | O(N log N) |
| Angle forces | numerical 6x | analytical 1x | analytical 1x | analytical |
| Water SHAKE | iterative ~5 iter | SETTLE 0 iter | SETTLE | SETTLE |
| Max dt (water) | 1 fs | 2 fs (SHAKE) | 4 fs (r-RESPA) | 2-4 fs |
| Folding throughput | 1 replica | 8 (REMD) | 64 (batch REMD) | 128+ |
