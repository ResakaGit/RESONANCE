# ADR-021: Bonded Force Architecture — Topology-Driven vs N^2

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** MOLECULAR_DYNAMICS track, sprints [MD-5](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD5_BONDED.md), [MD-6](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD6_TOPOLOGY.md)

## Contexto

El engine MD tiene fuerzas non-bonded (Coulomb + LJ) que operan sobre todos
los pares via cell list o brute force. Las fuerzas bonded (bonds, angles,
dihedrals) solo aplican a atomos conectados — necesitan un grafo de
conectividad, no un sweep N^2.

## Problema

Como integrar fuerzas bonded sin contaminar el pipeline de fuerzas non-bonded
y sin duplicar el patron de acumulacion.

## Decision

### D1: Separar bonded y non-bonded en sistemas distintos

Bonded forces iteran una lista de topologia (bond list, angle list, dihedral
list). Non-bonded forces usan cell list / brute force. Ambos escriben al
mismo acumulador de fuerzas. Total force = bonded + non-bonded (superposicion).

**Razon:** SRP. El sistema bonded no necesita saber sobre celdas o cutoff.
El sistema non-bonded no necesita saber sobre conectividad.

### D2: Math puro en `equations/bonded.rs`, topologia en `batch/topology.rs`

Las funciones de fuerza (harmonic_bond, harmonic_angle, dihedral) son puras:
parametros in → fuerza out. La estructura de topologia (listas de bonds,
angles, dihedrals, residuos) vive en `batch/topology.rs`.

**Razon:** Misma separacion que coulomb.rs (math) vs particle_forces.rs (system).

### D3: Topologia inmutable durante force computation

Bond formation/breaking ocurre en una fase separada (MorphologicalLayer),
no durante el calculo de fuerzas. La topologia es read-only en el force loop.

**Razon:** Evita race conditions y simplifica la logica.

### D4: Dihedrals en 3D desde el inicio

Aunque el batch simulator es 2D, las funciones de dihedral usan `[f32; 3]`.
El standalone LjWorld (para validacion) puede usar 3D directamente.

**Razon:** Los dihedrals son inherentemente 3D (angulo entre planos). No tiene
sentido implementar una version 2D que se descartaria en MD-7.

## Alternativas descartadas

**A. Unificar bonded + non-bonded en un solo sistema:** Viola SRP, complica
el force loop, y no permite optimizar independientemente.

**B. Almacenar topologia en EntitySlot:** Viola max 4 fields, mezcla hot/cold
data, y la topologia es per-molecule, no per-entity.

## Consecuencias

- Pipeline: `non_bonded_forces → bonded_forces → verlet_velocity_step`
- Topologia es un recurso externo al EntitySlot (cold data, DoD)
- Angle/dihedral inference automatica desde bond graph (MD-6)
