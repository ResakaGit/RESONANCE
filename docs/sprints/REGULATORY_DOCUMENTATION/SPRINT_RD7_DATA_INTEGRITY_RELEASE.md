# RD-7: Data Integrity + Audit Trail + Cybersecurity + Release Package

**Objetivo:** Completar el marco regulatorio con compliance de integridad de datos (21 CFR Part 11), plan de ciberseguridad, y paquete de release formal. Cierra el track.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Medio (5 documentos)
**Bloqueado por:** RD-5 (QMS necesita existir antes del release package)
**Desbloquea:** Release formal, deployment en entornos regulados

---

## Entregables

### 1. Evaluación de Cumplimiento Part 11 (`PART11_COMPLIANCE.md`)

**Estándar:** 21 CFR Part 11, EU Annex 11
**Contenido:**

**Controles aplicables y su implementación:**

| Requisito Part 11 | Implementación en RESONANCE | Estado |
|--------------------|-----------------------------|--------|
| §11.10(a) Validation | 3,113 tests + V&V documentation (RD-4) | ✅ |
| §11.10(b) Record generation | Deterministic output → reproducible records | ✅ |
| §11.10(c) Record protection | Git immutable history + Cargo.lock | ✅ |
| §11.10(d) Access limits | GitHub permissions (public read, controlled write) | ⚠️ |
| §11.10(e) Audit trails | Git log (immutable, timestamped, attributed) | ✅ |
| §11.10(f) Operational checks | `cargo test` must pass before merge | ✅ |
| §11.10(g) Authority checks | Git branch protection + PR review | ⚠️ |
| §11.10(h) Device checks | N/A (not a device, software-only) | N/A |
| §11.10(i) Training | Documented in CLAUDE.md (roles + rules) | ⚠️ |
| §11.10(k) Documentation | This regulatory documentation track | ✅ |
| §11.50 Signature manifestation | Git commit author + GPG signing (if enabled) | ⚠️ |
| §11.70 Signature linking | Commit hash links code change to author identity | ✅ |

**Exclusiones justificadas:**
- §11.10(h): No physical device — software-only simulation
- §11.30: Open systems — RESONANCE is open-source, not a closed GxP system

### 2. Política de Integridad de Datos (`DATA_INTEGRITY_POLICY.md`)

**Estándar:** ALCOA+ (WHO/EMA/FDA harmonized)

| Principio ALCOA+ | Implementación |
|-------------------|----------------|
| **A**ttributable | Git commit = author + timestamp + change |
| **L**egible | Markdown docs + Rust source = human-readable |
| **C**ontemporaneous | Commit timestamp = time of change |
| **O**riginal | Git blob = immutable original |
| **A**ccurate | 3,113 tests verify accuracy |
| **+Complete** | Git history = complete record of all changes |
| **+Consistent** | Deterministic build (Cargo.lock) = consistent artifacts |
| **+Enduring** | Git repository = permanent storage |
| **+Available** | GitHub public = always available |

### 3. Procedimiento de Audit Trail (`AUDIT_TRAIL.md`)

**Estándar:** 21 CFR Part 11 §11.10(e)
**Contenido:**
- Audit trail primario: `git log --all` (immutable, timestamped, attributed)
- Audit trail de simulación: deterministic seeds → reproducible outputs
- Audit trail de configuración: `Cargo.lock` + `assets/maps/*.ron` version-controlled
- Procedimiento de revisión: quarterly review of git log for anomalies
- Ejemplo de extracción:
  ```bash
  git log --oneline --since="2026-01-01" --until="2026-04-01" | wc -l
  git log --format="%H %ai %an %s" --since="2026-04-01" > audit_q2_2026.txt
  ```

### 4. Plan de Ciberseguridad (`CYBERSECURITY_PLAN.md`)

**Estándar:** FDA §524B FD&C Act, IMDRF Cybersecurity Guidance
**Contenido:**

**Threat model:**
- Simulation tampering: malicious modification of equations → wrong results
- Supply chain: compromised crate in dependency tree
- Data exfiltration: simulation outputs leaked (low risk — research data, not PHI)
- Denial of service: not applicable (offline tool, no network)

**Mitigations:**

| Threat | Mitigation | Implementation |
|--------|-----------|----------------|
| Code tampering | Git signed commits + SHA-256 integrity | Git infrastructure |
| Supply chain | `cargo audit` + pinned deps in Cargo.lock | Dev process |
| Unauthorized access | GitHub branch protection rules | Repository settings |
| Dependency CVE | RustSec advisory monitoring | `cargo audit` periodic |

**SBOM maintenance:** Updated with each dependency change (RD-3 SBOM).
**Patching cadence:** Security patches within 30 days of RUSTSEC advisory.
**No PHI/PII:** RESONANCE does not process, store, or transmit patient data.

### 5. Paquete de Release (`RELEASE_PACKAGE.md`)

**Estándar:** IEC 62304 §5.8
**Contenido:**

**Release criteria (all must be met):**
- [ ] `cargo test` — 0 failures
- [ ] `cargo check` — 0 warnings
- [ ] `cargo audit` — 0 known vulnerabilities
- [ ] All sprint closure criteria green (grep verification)
- [ ] Regulatory documentation complete (RD-1 through RD-7)
- [ ] Risk Management Report signed off (RD-2)
- [ ] Clinical Evaluation Report current (RD-6)

**Release artifacts:**
- Source code (Git tag + SHA-256)
- Binary builds (cargo build --release, per-platform)
- Documentation snapshot (docs/ at release commit)
- SBOM (generated from Cargo.lock at release commit)
- Test results (cargo test output at release commit)

**Release notes template:**
```markdown
## RESONANCE vX.Y.Z — Release Notes

**Date:** YYYY-MM-DD
**Commit:** [hash]
**Tests:** N passed, 0 failed
**Safety Class:** [A/B/C per SOFTWARE_SAFETY_CLASS.md]

### Changes
- [Sprint track summaries]

### Known Limitations
- [From LIMITATIONS_REPORT.md]

### Regulatory Status
- [Current classification and pending items]
```

**Post-release:**
- Tag Git commit with version
- Archive release documentation in `docs/releases/vX.Y.Z/`
- Monitor for post-market feedback (GitHub issues)

---

## Scope definido

**Entra:** 5 documentos en `docs/regulatory/07_release/`
**NO entra:** Implementar GPG signing, CI/CD pipeline, automated cargo audit

## Criterios de cierre

- [ ] Part 11 evaluation covers all §11.10 subsections with disposition
- [ ] ALCOA+ policy maps each principle to implementation
- [ ] Audit trail procedure includes extraction commands
- [ ] Cybersecurity plan covers IMDRF threat categories
- [ ] Release package defines verifiable go/no-go criteria
