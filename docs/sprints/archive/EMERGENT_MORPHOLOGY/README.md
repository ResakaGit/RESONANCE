# Track: EMERGENT_MORPHOLOGY — De Tubos a Vertebrados

2D radial energy field → bilateral organisms with extremities and joints.
100% emergente desde los 8 axiomas. Zero labels top-down.

**Estado:** ✅ ARCHIVADO (2026-03-27) — 4/4 sprints completados

Design doc: [`docs/design/EMERGENT_MORPHOLOGY.md`](../../design/EMERGENT_MORPHOLOGY.md)

---

## Sprints (4)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [EM-1](SPRINT_EM1_RADIAL_FIELD_EQUATIONS.md) | Radial Field Equations | Medio | — | `radial_field.rs`: diffusion 2D, peak detection, gradients, joints |
| [EM-2](SPRINT_EM2_ARENA_INTEGRATION.md) | Arena Integration | Medio | EM-1 | EntitySlot 2D fields, `internal_diffusion` 2D, pipeline wired |
| [EM-3](SPRINT_EM3_APPENDAGE_INFERENCE.md) | Appendage Inference | Alto | EM-2 | Peaks → sub-meshes, bilateral emergence, `creature_builder` 2D |
| [EM-4](SPRINT_EM4_JOINT_ARTICULATION.md) | Joint Articulation | Alto | EM-3 | Valley detection → joints, segmented appendages, visual diversity |

---

## Dependency chain

```
EM-1 (equations) → EM-2 (arena) → EM-3 (geometry) → EM-4 (joints)
```

Serial. Each sprint builds on the previous.

---

## Axiom compliance

| Axiom | How respected |
|-------|---------------|
| 1. Everything is qe | Radial field nodes are qe. Peaks/valleys are qe concentrations. |
| 3. Competition × interference | Appendage shape modulated by interference at peak. |
| 4. Dissipation | 2D diffusion = entropy. Energy spreads from peaks. |
| 5. Conservation | `radial_total(field) == entity.qe` invariant. |
| 6. Emergence | Bilateral symmetry from isotropic init. Peaks not programmed. |
| 7. Distance attenuation | Diffusion only between adjacent nodes. |
| 8. Oscillatory | Frequency field per node. Color varies by local freq. |

## Zero hardcoded labels

No `Head`, `Leg`, `Arm`, `Tail` enums. No `Quadruped`, `Biped` templates.
No `bilateral_quadruped_attachments()` hardcoded positions.
Morphology is: peaks → appendages, valleys → joints, symmetry → bilateral.
