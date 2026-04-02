# RI-1: CI/CD Pipeline + Branch Protection

**Objetivo:** Establecer la infraestructura minima de integracion continua y proteccion de rama para que cada cambio a `main` sea verificado automaticamente. Cierra el gap #4 (no CI/CD) y habilita parcialmente #3 (GPG via CI) y #1 (approval via PR reviews).

**Estado:** PENDIENTE
**Esfuerzo:** Medio (5 entregables, configuracion GitHub + workflows)
**Bloqueado por:** --
**Desbloquea:** RI-2, RI-3

---

## Entregables

### 1. GitHub Actions CI Workflow (`.github/workflows/ci.yml`)

**Trigger:** push a `main` + every PR targeting `main`
**Jobs:**
- `check`: `cargo check --workspace` (compilacion rapida)
- `test`: `cargo test --workspace` (3,113 tests, timeout 10 min)
- `clippy`: `cargo clippy --workspace -- -D warnings` (zero warnings policy)
- `audit`: `cargo audit` (fail on CVSS >= 7.0)
- `fmt`: `cargo fmt --check` (formatting)

**Matrix:** stable Rust (pinned via `rust-toolchain.toml`)
**Cache:** `actions/cache` para `~/.cargo/registry` + `target/`

**Evidencia de cierre:** CI badge verde en un PR merged.

### 2. Branch Protection en `main`

**Configuracion via GitHub Settings > Branches > Branch protection rules:**
- Require pull request before merging (min 1 approval)
- Require status checks to pass: `check`, `test`, `clippy`, `audit`
- Require branches to be up to date before merging
- Do not allow bypassing the above settings (incluye admins)

**Evidencia de cierre:** push directo a `main` bloqueado; screenshot o `gh api` output.

### 3. `cargo audit` Integration

**Herramienta:** `cargo-audit` (ya en ecosystem Rust, no crate nuevo)
**Threshold:** fail CI si advisory con CVSS >= 7.0
**Alternativa:** `cargo deny` si se necesita policy mas granular (futuro)

**Evidencia de cierre:** `cargo audit` ejecuta en CI sin advisories criticos.

### 4. `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

Pin de version exacta del toolchain para reproducibilidad entre dev machines y CI.

**Evidencia de cierre:** archivo existe en raiz y CI lo usa.

### 5. Dependabot / RustSec Monitoring

**Opcion A (recomendada):** GitHub Dependabot alerts (zero config, built-in)
- Activar Dependabot alerts en Settings > Code security
- Opcionalmente: `.github/dependabot.yml` para Cargo updates semanales

**Opcion B:** RustSec advisory-db via `cargo audit` en CI (ya cubierto en entregable 3)

**Evidencia de cierre:** Dependabot alerts habilitado en repo settings.

---

## Scope definido

**Entra:** `.github/workflows/ci.yml`, `rust-toolchain.toml`, branch protection config, Dependabot config
**NO entra:** cambios a `src/`, GPG signing enforcement (RI-2), deploy/release workflows, Docker

## Criterios de cierre

- [ ] `.github/workflows/ci.yml` existe y ejecuta 5 jobs (check, test, clippy, audit, fmt)
- [ ] CI pasa verde en al menos 1 PR merged
- [ ] Branch protection activa: PR required + CI required + 1 review
- [ ] `rust-toolchain.toml` en raiz con toolchain pinned
- [ ] Dependabot alerts habilitado en Settings > Code security
- [ ] Push directo a `main` rechazado (verificar con intento fallido o `gh api`)
