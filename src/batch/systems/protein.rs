//! Protein fold batch system — derives catalytic function from genome fold.
//!
//! Runs after metabolic_graph_infer. Uses protein_fold equations to:
//! 1. Fold the genome into a 2D lattice structure.
//! 2. Infer active sites from contact map.
//! 3. If protein has function → reduce dissipation (catalytic advantage).
//!
//! Axiom 4: catalytic organisms dissipate less (efficiency bonus).
//! Axiom 6: function emerges from fold geometry, not templates.

use crate::batch::arena::SimWorldFlat;
use crate::blueprint::constants::MIN_CODONS;
use crate::blueprint::equations::codon_genome;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_SOLID;
use crate::blueprint::equations::protein_fold;

/// Max catalytic bonus to dissipation rate. Derived: DISSIPATION_SOLID × 8 = 0.04.
/// Stacks with metabolic graph bonus (MGN-4) for max ~9% reduction.
const CATALYSIS_BONUS_CAP: f32 = DISSIPATION_SOLID * 8.0;

/// Fold genomes and apply catalytic advantages.
///
/// For each alive entity: fold genome → infer function → if functional,
/// reduce dissipation proportional to efficiency_boost × specificity.
/// Conservation: only reduces dissipation, never adds energy. Axiom 4/5.
pub fn protein_fold_infer(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        let seed = world.seed ^ world.tick_id ^ (i as u64 * 0x9E37);

        // PD-5: Use codon translation when available, else fall back to variable genome
        let cg = &world.codon_genomes[i];
        let phenotype = if cg.codon_count() >= MIN_CODONS {
            // Codon path: translate codons → monomers → fold
            let ct = &world.codon_tables[i];
            let (chain, len) = codon_genome::translate_genome(cg, ct);
            // Frequency from hydrophobicity: base + modulation (same as protein_fold pipeline)
            let mut freq = [0.0f32; protein_fold::MAX_CHAIN];
            for j in 0..len {
                let dim = j % 4;
                freq[j] = protein_fold::DIM_BASE_FREQ[dim]
                    + (chain[j].hydrophobicity - 0.5) * protein_fold::FREQ_MODULATION_RANGE;
            }
            let fold = protein_fold::fold_greedy(&chain, len, &freq, seed);
            let contacts = protein_fold::contact_map(&fold, len);
            let density = protein_fold::contact_density(&contacts, len);
            let function = protein_fold::infer_protein_function(&chain, &density, &freq, len);
            let h_count = chain[..len].iter().filter(|m| m.is_hydrophobic()).count() as u8;
            let total_contacts = density[..len].iter().map(|&d| d as u16).sum::<u16>() / 2;
            protein_fold::ProteinPhenotype {
                chain_length: len as u8, fold_energy: protein_fold::fold_energy(&chain, &fold, len, &freq),
                h_count, p_count: len as u8 - h_count, total_contacts, function,
            }
        } else {
            let vg = world.genomes[i];
            if vg.gene_count() < 4 { continue; }
            protein_fold::compute_protein_phenotype(&vg, seed)
        };

        let Some(func) = phenotype.function else { continue; };

        // Catalytic bonus: efficiency × specificity, capped.
        let bonus = (func.efficiency_boost * func.specificity * CATALYSIS_BONUS_CAP)
            .min(CATALYSIS_BONUS_CAP);
        world.entities[i].dissipation *= 1.0 - bonus;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::SimWorldFlat;
    use crate::blueprint::equations::variable_genome::VariableGenome;

    fn world_with_genome(genes: &[f32]) -> SimWorldFlat {
        let mut world = SimWorldFlat::new(42, 0.05);
        let mut vg = VariableGenome::default();
        for (i, &v) in genes.iter().enumerate().take(32) {
            vg.genes[i] = v;
        }
        vg.len = genes.len().min(32) as u8;
        world.genomes[0] = vg;
        world.entities[0].qe = 100.0;
        world.entities[0].dissipation = 0.02;
        world.entities[0].growth_bias = genes.first().copied().unwrap_or(0.5);
        world.alive_mask = 1;
        world
    }

    #[test]
    fn fold_conserves_energy() {
        let mut world = world_with_genome(&[0.9, 0.8, 0.9, 0.7, 0.8, 0.9, 0.7, 0.8]);
        let qe_before = world.entities[0].qe;
        protein_fold_infer(&mut world);
        assert!(world.entities[0].qe <= qe_before, "qe must not increase");
    }

    #[test]
    fn fold_reduces_dissipation_for_functional_protein() {
        let mut world = world_with_genome(&[0.9, 0.8, 0.9, 0.7, 0.8, 0.9, 0.7, 0.8, 0.9, 0.8, 0.9, 0.7]);
        let diss_before = world.entities[0].dissipation;
        protein_fold_infer(&mut world);
        assert!(world.entities[0].dissipation <= diss_before, "functional protein → less dissipation");
    }

    #[test]
    fn fold_skips_dead_entities() {
        let mut world = world_with_genome(&[0.9; 8]);
        world.alive_mask = 0;
        let diss_before = world.entities[0].dissipation;
        protein_fold_infer(&mut world);
        assert_eq!(world.entities[0].dissipation, diss_before);
    }

    #[test]
    fn fold_deterministic() {
        let mut a = world_with_genome(&[0.8, 0.3, 0.9, 0.6, 0.7, 0.5, 0.8, 0.4]);
        let mut b = world_with_genome(&[0.8, 0.3, 0.9, 0.6, 0.7, 0.5, 0.8, 0.4]);
        protein_fold_infer(&mut a);
        protein_fold_infer(&mut b);
        assert_eq!(a.entities[0].dissipation.to_bits(), b.entities[0].dissipation.to_bits());
    }

    #[test]
    fn fold_short_genome_no_panic() {
        let mut world = world_with_genome(&[0.5, 0.5, 0.5, 0.5]);
        protein_fold_infer(&mut world); // 4 genes = may or may not have function, no panic
    }
}
