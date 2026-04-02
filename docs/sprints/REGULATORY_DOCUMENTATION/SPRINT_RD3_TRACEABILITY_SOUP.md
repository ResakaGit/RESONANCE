# RD-3: Traceability + SOUP Analysis + SBOM + Configuration Management

**Objetivo:** Establecer la cadena de trazabilidad completa (requisitos → diseño → implementación → tests → riesgos) y documentar formalmente todas las dependencias de terceros (SOUP).

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Medio (4 documentos, mayormente mecánico a partir de datos existentes)
**Bloqueado por:** RD-1 (SRS provee los requisitos a trazar)
**Desbloquea:** RD-4 (V&V necesita trazabilidad para mapear coverage)

---

## Entregables

### 1. Matriz de Trazabilidad (`TRACEABILITY_MATRIX.md`)

**Estándar:** IEC 62304 §5.7.4 + GAMP 5
**Estructura:**

```
Requisito (SRS) → Diseño (Architecture) → Módulo (src/) → Tests → Riesgo (ISO 14971)
```

**Ejemplo de trazabilidad forward:**

| Req | Diseño | Implementación | Tests | Riesgo |
|-----|--------|---------------|-------|--------|
| RF-01 (emergent life) | ARCHITECTURE.md §14 layers | `src/layers/`, `src/simulation/` | 2,800+ system tests | H-02 |
| RF-02 (drug-pathway) | `docs/design/PATHWAY_INHIBITOR.md` | `src/blueprint/equations/pathway_inhibitor.rs` | 32 tests | H-01, H-03 |
| RF-04 (Bozic 2013) | Paper §Exp 7 | `src/bin/bozic_validation.rs` | 10-seed validation | H-03 |
| RF-06 (determinism) | ARCHITECTURE.md §Determinism | `src/blueprint/equations/determinism.rs` | hash_f32_slice tests | H-04 |
| RF-09 (conservation) | Axiom 5 | `src/simulation/metabolic/basal_drain.rs` | `tests/property_conservation.rs` | H-02 |

**Bidirectional:** Cada test debe mapear a ≥1 requisito. Cada requisito debe tener ≥1 test.

### 2. Análisis SOUP (`SOUP_ANALYSIS.md`)

**Estándar:** IEC 62304 §5.3.3 (Software of Unknown Provenance)
**Contenido:** Evaluación formal de cada dependencia en Cargo.toml.

| Crate | Version | Purpose | Anomaly Risk | Published CVEs | Mitigation |
|-------|---------|---------|-------------|----------------|------------|
| bevy | 0.15 | ECS engine + rendering | High (complex) | Check crates.io/advisory-db | Pin version, monitor RUSTSEC |
| glam | 0.29 | Math primitives (Vec2, Vec3) | Low (pure math) | None known | Pin, unit tests verify |
| rayon | 1.10 | Parallel batch simulation | Medium (threading) | None known | Used only in batch/, not runtime |
| serde | 1 | Serialization | Low (stable, audited) | None known | Standard Rust ecosystem |
| ron | 0.8 | Config files (.ron assets) | Low | None known | Asset loading only |
| noise | 0.9 | Terrain noise generation | Low (pure math) | None known | Used only in topology/ |
| oxidized_navigation | 0.12 | Pathfinding (A*) | Medium (complex) | None known | Used only in pathfinding/ |
| parry3d | 0.17 | Collision detection | Medium | None known | Used only in collision backend |
| bevy_egui | 0.31 | UI rendering | Medium | None known | Used only in dashboard/lab |
| egui_plot | 0.29 | Charts | Low | None known | Used only in dashboard |
| fxhash | 0.2 | Fast hashing | Low (pure math) | None known | Determinism verified by tests |
| tracing | 0.1 | Logging | Low | None known | Standard ecosystem |
| bytemuck | 1.25 | GPU buffer layout | Low (unsafe justified) | None known | 4 unsafe impls, all verified |
| minifb | 0.27 (optional) | Headless framebuffer | Low | None known | Optional feature only |

**Dev-dependencies (no runtime impact):** criterion, naga, pollster, proptest, wgpu — no SOUP risk.

**Proceso:** Para cada crate, verificar [RustSec Advisory Database](https://rustsec.org/) y `cargo audit`.

### 3. Software Bill of Materials (`SBOM.md`)

**Estándar:** FDA Cybersecurity Guidance, NTIA SBOM requirements
**Formato:** CycloneDX-compatible markdown (exportable a JSON)
**Contenido:**
- Componente, versión, licencia, hash, fuente
- Generado desde `cargo metadata` + `Cargo.lock`
- Incluye transitive dependencies

**Comando de generación:**
```bash
cargo tree --depth 1 --format "{p} {l}" > sbom_direct.txt
cargo tree --all-features > sbom_full.txt
```

### 4. Gestión de Configuración (`CONFIGURATION_MANAGEMENT.md`)

**Estándar:** IEC 62304 §8
**Contenido:**
- Configuration items: código fuente, Cargo.toml, Cargo.lock, assets/maps/*.ron, docs/
- Versioning: Git (commit hash = item version), semver en Cargo.toml
- Branch strategy: main-only (trunk-based per commit history)
- Build reproducibility: `cargo build --release` + Cargo.lock = deterministic binary
- Change control: sprint-based (docs/sprints/), PR review (Verificador role)

---

## Scope definido

**Entra:** 4 documentos en `docs/regulatory/03_traceability/`
**NO entra:** Cargo audit automatizado (CI pipeline — fuera de scope documental)

## Criterios de cierre

- [ ] Traceability matrix cubre ≥10 requisitos de SRS con forward + backward links
- [ ] SOUP analysis cubre 14 runtime dependencies + risk assessment
- [ ] SBOM incluye direct + transitive dependencies con versiones y licencias
- [ ] Configuration management referencia Git, Cargo.lock, sprint history
