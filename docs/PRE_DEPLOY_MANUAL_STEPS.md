# Pre-Deploy Manual Steps — RESONANCE

Acciones manuales que NO se pueden automatizar desde código y requieren intervención humana en GitHub Settings, terminal local, o servicios externos.

**Estado:** Pendiente post-push de `8b07373`
**Prerequisito:** Los archivos de infraestructura ya están en `main` (CI workflow, dependabot, issue templates, rust-toolchain.toml)

---

## 1. Branch Protection en `main`

**Dónde:** GitHub → Settings → Branches → Add rule

**Branch name pattern:** `main`

| Setting | Valor | Por qué |
|---------|-------|---------|
| Require a pull request before merging | ✅ ON | Impide push directo. Cierra gap #1 (approval) |
| Required approvals | 1 | Mínimo 1 review (Verificador o Observador) |
| Require status checks to pass before merging | ✅ ON | CI debe pasar para merge |
| Status checks required | `Check`, `Test`, `Clippy`, `Audit`, `Format` | Los 5 jobs del workflow `ci.yml` |
| Require branches to be up to date before merging | ✅ ON | Evita merge de código stale |
| Require signed commits | ✅ ON (activar después del paso 2) | 21 CFR Part 11 §11.50 |
| Do not allow bypassing the above settings | ✅ ON | Incluye admins — nadie bypasea |
| Allow force pushes | ❌ OFF | Protege historia inmutable |
| Allow deletions | ❌ OFF | Protege la rama |

**Verificación:**
```bash
# Intentar push directo (debe fallar):
git commit --allow-empty -m "test" && git push
# Expected: remote: error: GH006: Protected branch update failed
```

**Nota:** Una vez activado, TODO cambio a `main` requiere PR. Esto incluye hotfixes — crear branch temporal, PR, merge.

---

## 2. GPG / SSH Commit Signing

**Dónde:** Terminal local + GitHub Settings

### Opción A: SSH Signing (recomendada — más simple)

```bash
# 1. Generar key (si no tienes una)
ssh-keygen -t ed25519 -C "tu@email.com" -f ~/.ssh/id_ed25519_signing

# 2. Configurar git para usar SSH signing
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true

# 3. Verificar
git commit --allow-empty -m "test: signed commit"
git log --show-signature -1
# Expected: "Good signature"
```

**GitHub:** Settings → SSH and GPG keys → New SSH key → Key type: **Signing Key** → Pegar el contenido de `~/.ssh/id_ed25519_signing.pub`

### Opción B: GPG Signing (tradicional)

```bash
# 1. Generar key
gpg --full-generate-key
# Seleccionar: RSA 4096, no expiration, tu nombre y email de GitHub

# 2. Obtener key ID
gpg --list-secret-keys --keyid-format=long
# Copiar el ID después de "sec rsa4096/"

# 3. Configurar git
git config --global user.signingkey <KEY_ID>
git config --global commit.gpgsign true

# 4. Exportar public key
gpg --armor --export <KEY_ID>
# Copiar output
```

**GitHub:** Settings → SSH and GPG keys → New GPG key → Pegar el armor block

### Verificación post-setup

```bash
# Nuevo commit firmado
git commit --allow-empty -m "test: verify signing"
git log --show-signature -1

# En GitHub: el commit debe mostrar "Verified" badge
```

**Nota:** Después de configurar signing, activar "Require signed commits" en branch protection (paso 1).

---

## 3. Activar Dependabot Alerts

**Dónde:** GitHub → Settings → Code security and analysis

| Setting | Valor |
|---------|-------|
| Dependency graph | ✅ Enable |
| Dependabot alerts | ✅ Enable |
| Dependabot security updates | ✅ Enable |

El archivo `.github/dependabot.yml` ya está en el repo — esto activa las alertas del lado de GitHub.

**Verificación:** GitHub → Security → Dependabot → debe mostrar "No known vulnerabilities" o listar advisories

---

## 4. Verificar que CI ejecuta

**Dónde:** GitHub → Actions

Después del push de `8b07373`, el workflow CI debería ejecutar automáticamente.

**Si no ejecuta:**
1. Verificar que GitHub Actions está habilitado: Settings → Actions → General → "Allow all actions"
2. Verificar que el workflow está en la rama correcta: `.github/workflows/ci.yml` en `main`

**Primera ejecución esperada:** ~5-8 min (test job es el más largo)

**Posibles fallos en primera ejecución:**

| Job | Posible fallo | Solución |
|-----|--------------|----------|
| `test` | Bevy requiere libs de sistema en Ubuntu | Agregar `sudo apt-get install -y libasound2-dev libudev-dev` antes de `cargo test` |
| `clippy` | Warnings nuevos en Rust stable más reciente | Fix warnings o ajustar CI |
| `audit` | Advisory en dependencia | Evaluar severity — si CVSS < 7.0, aceptable |
| `fmt` | Formatting drift | `cargo fmt` local y push |

**Fix probable para Bevy en Ubuntu (si test falla):**

Editar `.github/workflows/ci.yml`, agregar antes de `cargo test` y `cargo check` y `cargo clippy`:
```yaml
      - name: Install system dependencies
        run: sudo apt-get update && sudo apt-get install -y libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

---

## 5. Crear Milestone Q3-2026-REVIEW

**Dónde:** GitHub → Issues → Milestones → New milestone

| Campo | Valor |
|-------|-------|
| Title | Q3-2026-REVIEW |
| Due date | 2026-09-30 |
| Description | First quarterly regulatory review. Scope: RD-2.* (risk file), RD-3.2 (SOUP), RD-3.3 (SBOM), RD-2.7 (post-production monitoring). Template: `docs/regulatory/05_quality_system/QUARTERLY_REVIEW_TEMPLATE.md` |

**Verificación:** Milestone aparece en Issues → Milestones con fecha 2026-09-30

---

## 6. Crear Labels para CCB Workflow

**Dónde:** GitHub → Issues → Labels → New label

| Label | Color | Descripción |
|-------|-------|-------------|
| `ccb-review` | `#7057FF` (purple) | Requires Change Control Board review |
| `regulatory` | `#0E8A16` (green) | Related to regulatory documentation |
| `axiom-impact` | `#D93F0B` (red) | May affect foundational axioms |
| `safety-class` | `#B60205` (dark red) | May affect safety classification |

**Verificación:** Labels aparecen en Issues → Labels

---

## 7. Configurar Notifications

**Dónde:** GitHub → Settings → Notifications (personal)

Recomendado:
- ✅ Dependabot alerts → Email
- ✅ PR reviews → Email
- ✅ CI failures → Email (o GitHub mobile)

---

## 8. Primer PR de Prueba (Validación End-to-End)

Una vez completados los pasos 1-6, validar el flujo completo:

```bash
# 1. Crear branch
git checkout -b test/validate-ci

# 2. Hacer cambio trivial
echo "" >> docs/regulatory/REVIEW_LOG.md
git add docs/regulatory/REVIEW_LOG.md
git commit -m "test: validate CI pipeline"

# 3. Push y crear PR
git push -u origin test/validate-ci
gh pr create --title "test: validate CI pipeline" --body "Validación end-to-end del pipeline CI + branch protection"

# 4. Verificar:
#    - CI ejecuta 5 jobs
#    - PR requiere review
#    - PR requiere signed commit (si paso 2 completo)
#    - Merge solo posible cuando CI verde + 1 approval

# 5. Cleanup después de validar
gh pr close --delete-branch
git checkout main
```

---

## Checklist de Verificación Final

```
[ ] 1. Branch protection activa en main
      [ ] PR required
      [ ] Status checks required (5 jobs)
      [ ] Signed commits required
      [ ] Bypass disabled
[ ] 2. Commit signing configurado
      [ ] Key generada (SSH o GPG)
      [ ] Key subida a GitHub
      [ ] git config commit.gpgsign = true
      [ ] Commits muestran "Verified" en GitHub
[ ] 3. Dependabot alerts habilitado
      [ ] Dependency graph ON
      [ ] Alerts ON
      [ ] Security updates ON
[ ] 4. CI ejecutando
      [ ] Workflow aparece en Actions
      [ ] 5 jobs ejecutan
      [ ] Verde (o fallos diagnosticados)
[ ] 5. Milestone Q3-2026-REVIEW creado
[ ] 6. Labels CCB creados (4 labels)
[ ] 7. Notifications configuradas
[ ] 8. PR de prueba validado end-to-end
```

---

## Orden Recomendado de Ejecución

```
1. Dependabot alerts (30 seg)
2. Verificar CI ejecuta (esperar ~8 min)
3. Fix CI si falla (Bevy deps en Ubuntu)
4. GPG/SSH signing (5 min)
5. Branch protection (2 min, activar signed commits DESPUÉS de 4)
6. Labels + Milestone (2 min)
7. PR de prueba (5 min)
```

**Tiempo total estimado:** 20-30 minutos (sin contar espera de CI)

---

*Este documento se puede eliminar después de completar todos los pasos. Su contenido queda como evidencia en el historial de Git.*
