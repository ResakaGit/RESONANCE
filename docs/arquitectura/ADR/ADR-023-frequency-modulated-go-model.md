# ADR-023: Frequency-Modulated Go Model — The Original Contribution

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** MOLECULAR_DYNAMICS track, sprints [MD-15](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD15_GO_MODEL.md), [MD-17](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD17_FOLD_VALIDATE.md)

## Contexto

Los Go models clasicos (Taketomi, Ueda & Go, 1975) definen contactos nativos
desde una estructura PDB y asignan fuerza de atraccion identica a todos los
pares nativos. No-nativos solo repelen.

## La contribucion original de Resonance

Cada residuo tiene una frecuencia caracteristica (Axiom 8). Los contactos
nativos son pares frecuencia-coherentes. Los contactos no-nativos tienen
frecuencias desfasadas y por tanto menor atraccion.

```
E_contact(i,j) = epsilon * alignment(f_i, f_j) * [5*(sigma/r)^12 - 6*(sigma/r)^10]
alignment(f_i, f_j) = exp(-0.5 * ((f_i - f_j) / COHERENCE_BANDWIDTH)^2)
```

**Esto NO existe en la literatura.**

## Decision

### D1: Implementar 3 estrategias de asignacion de frecuencias

| Estrategia | Fuente | Pros | Contras |
|-----------|--------|------|---------|
| **A: Amino acid type** | 20 frecuencias base por tipo | Determinista, reproducible | Arbitrario |
| **B: Structure-derived** | Optimizar para maximizar coherencia nativa | Garantiza funcionar | Reverse engineering |
| **C: Evolutionary** | Batch genetic harness evoluciona frecuencias | Mas "Resonance-native" | Lento |

Implementar las tres. Default: A. Comparar las tres en MD-17.

### D2: Coherencia como observable, no solo energia

El Go model clasico solo mide Q (fraccion de contactos nativos). El modelo
Resonance agrega **coherencia** como observable independiente:

```
coherence = mean(alignment(f_i, f_j)) over native contacts
```

Esto permite distinguir:
- Folded + coherent (nativo)
- Folded + incoherent (mis-folded — contactos correctos pero frecuencias mal)
- Unfolded + incoherent (desplegado)

### D3: Drug binding extiende naturalmente

`pathway_inhibitor.rs` ya modela binding por alineamiento de frecuencia.
El Go model usa el mismo framework — un farmaco que desplaza la frecuencia
de un residuo desestabiliza sus contactos nativos.

## Diferenciadores publicables

1. Folding = frequency synchronization (not just energy minimization)
2. Misfolded states detectable by coherence (not just RMSD)
3. Mutations = frequency shifts → quantifiable ΔΔG
4. Drug binding → folding stability → unified framework

## Consecuencias

- `equations/go_model.rs`: go_native_potential, go_axiom8_potential, native_contact_map
- `batch/ff/pdb.rs`: minimal PDB parser (C-alpha only)
- `batch/systems/go_forces.rs`: force computation
- Validacion en MD-17: villin headpiece < 5 A RMSD
