# CT-3: Molecular Bridge — Organismo → Proteínas

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** CT-1
**ADR:** ADR-036 §D4 (S3→S4)

## Objetivo

Conectar la escala ecológica (S3) con la molecular (S4). Al hacer zoom en un
organismo, ver sus proteínas plegándose con Go model + REMD.

## Precondiciones

- CT-1 completado (zoom engine con inference)
- Go model funcional (`blueprint/equations/go_model.rs`)
- REMD paralelo funcional (`batch/systems/remd.rs`)
- fold_go validado (Q=0.72, Rg match)

## Entregables

### E1: `ecological_to_molecular.rs` — bridge S3→S4

```rust
// src/cosmic/bridges/ecological_to_molecular.rs

/// Inferir proteínas de un organismo a partir de su estado ECS.
///
/// - Número de proteínas ∝ organism.qe^Kleiber
/// - Frecuencia de cada residuo derivada de organism.freq (Axiom 8)
/// - Tamaño de proteína ∝ log(organism.qe)
pub fn infer_proteome(
    organism_qe: f64,
    organism_freq: f64,
    organism_age: u64,
    seed: u64,
) -> Vec<ProteinSpec>;

pub struct ProteinSpec {
    pub n_residues: usize,       // 20-100
    pub sequence: Vec<u8>,       // amino acid types derivados de freq
    pub qe_budget: f64,          // energía asignada a esta proteína
}

/// Construir GoTopology + run REMD para una proteína inferida.
pub fn fold_protein(spec: &ProteinSpec, bandwidth: f64) -> FoldingResult;
```

### E2: Zoom-in visual

Al hacer zoom en un organismo en S3:
1. Inferir proteome via `infer_proteome`
2. Mostrar proteínas como esferas conectadas (C-alpha backbone)
3. Correr REMD en background (paralelo, ya implementado)
4. Actualizar visualización con best structure en tiempo real

### E3: Zoom-out agregación

Al zoom-out, el estado de folding (Q, Rg, coherence) se agrega al organismo
como un "health score" que afecta su viability en S3.

## Tasks

- [ ] Crear `src/cosmic/bridges/ecological_to_molecular.rs`
- [ ] `infer_proteome`: derivar proteínas de observables del organismo
- [ ] `fold_protein`: wrapper sobre `remd::run_remd` con config apropiada
- [ ] Test: proteínas inferidas tienen `sum(qe) <= organism.qe`
- [ ] Test: secuencias derivadas de frecuencia son deterministas por seed
- [ ] Test: fold_protein retorna Q > 0 para proteínas de ≥20 residuos
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Organismo con qe=1000 produce ~3-8 proteínas (Kleiber scaling)
2. Cada proteína tiene secuencia derivada de freq del organismo
3. REMD corre y produce Q > 0 para cada proteína
4. `sum(protein.qe) <= organism.qe` (Pool Invariant)
5. Determinista por seed
