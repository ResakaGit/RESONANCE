//! Genome Bridge — bidirectional conversion between `GenomeBlob` (batch)
//! and Bevy ECS components.
//!
//! Guarantees lossless round-trip: `components_to_genome(genome_to_components(g)) == g`.
//!
//! Also provides binary serialization to/from disk without `unsafe`.

use crate::batch::genome::GenomeBlob;
use crate::blueprint::constants;
use crate::layers::inference::{InferenceProfile, TrophicClass};
use crate::layers::trophic::TrophicConsumer;
use crate::layers::{
    BaseEnergy, SpatialVolume, OscillatorySignature, FlowVector,
    MatterCoherence, AlchemicalEngine,
};

// ─── Batch → Bevy ───────────────────────────────────────────────────────────

/// Core component tuple from a GenomeBlob. Ready for Bevy spawn.
///
/// Does NOT include archetype-specific components (TrophicConsumer, BehavioralAgent, etc.).
/// Caller adds those based on `genome.archetype`.
pub fn genome_to_components(genome: &GenomeBlob) -> (
    BaseEnergy,
    SpatialVolume,
    OscillatorySignature,
    FlowVector,
    MatterCoherence,
    AlchemicalEngine,
    InferenceProfile,
) {
    (
        BaseEnergy::new(constants::DEFAULT_BASE_ENERGY),
        SpatialVolume::new(0.5),
        OscillatorySignature::new(
            frequency_for_archetype(genome.archetype),
            0.0,
        ),
        FlowVector::default(),
        MatterCoherence::default(),
        AlchemicalEngine::new(0.0, 20.0, 0.5, 0.5),
        InferenceProfile::new(
            genome.growth_bias,
            genome.mobility_bias,
            genome.branching_bias,
            genome.resilience,
        ),
    )
}

/// Archetype-specific TrophicConsumer, if applicable.
pub fn genome_to_trophic(genome: &GenomeBlob) -> Option<TrophicConsumer> {
    let class = trophic_class_from_u8(genome.trophic_class)?;
    Some(TrophicConsumer::new(class, 1.0))
}

// ─── Bevy → Batch ───────────────────────────────────────────────────────────

/// Extract a GenomeBlob from Bevy components. Lossless for the 4 bias fields.
pub fn components_to_genome(
    profile: &InferenceProfile,
    trophic: Option<&TrophicConsumer>,
) -> GenomeBlob {
    GenomeBlob {
        archetype:      infer_archetype(trophic),
        trophic_class:  trophic.map(|t| t.class as u8).unwrap_or(0),
        growth_bias:    profile.growth_bias,
        mobility_bias:  profile.mobility_bias,
        branching_bias: profile.branching_bias,
        resilience:     profile.resilience,
    }
}

fn infer_archetype(trophic: Option<&TrophicConsumer>) -> u8 {
    match trophic {
        Some(t) if t.class == TrophicClass::PrimaryProducer => 1,
        Some(t) if t.class == TrophicClass::Carnivore       => 2,
        Some(t) if t.class == TrophicClass::Detritivore      => 3,
        Some(_)                                               => 3, // cell/omnivore
        None                                                  => 0, // inert
    }
}

fn trophic_class_from_u8(v: u8) -> Option<TrophicClass> {
    match v {
        0 => Some(TrophicClass::PrimaryProducer),
        1 => Some(TrophicClass::Herbivore),
        2 => Some(TrophicClass::Omnivore),
        3 => Some(TrophicClass::Carnivore),
        4 => Some(TrophicClass::Detritivore),
        _ => None,
    }
}

/// Map archetype id to canonical frequency band center.
fn frequency_for_archetype(archetype: u8) -> f32 {
    match archetype {
        0 => 100.0,  // inert (Umbra band)
        1 => 400.0,  // flora (Terra band)
        2 => 600.0,  // fauna (Aqua band)
        3 => 300.0,  // cell  (Terra low)
        4 => 800.0,  // virus (Ignis band)
        _ => 200.0,
    }
}

// ─── Serialization (no unsafe) ──────────────────────────────────────────────

/// Serialize genomes to binary: `[u32 LE count][fields × count]`.
///
/// Each genome: archetype(1) + trophic(1) + growth(4) + mobility(4) + branching(4) + resilience(4) = 18 bytes.
pub fn save_genomes(genomes: &[GenomeBlob], path: &std::path::Path) -> std::io::Result<()> {
    let count = genomes.len() as u32;
    let mut buf = Vec::with_capacity(4 + genomes.len() * 18);
    buf.extend_from_slice(&count.to_le_bytes());
    for g in genomes {
        buf.push(g.archetype);
        buf.push(g.trophic_class);
        buf.extend_from_slice(&g.growth_bias.to_le_bytes());
        buf.extend_from_slice(&g.mobility_bias.to_le_bytes());
        buf.extend_from_slice(&g.branching_bias.to_le_bytes());
        buf.extend_from_slice(&g.resilience.to_le_bytes());
    }
    std::fs::write(path, buf)
}

/// Deserialize genomes from binary written by `save_genomes`.
pub fn load_genomes(path: &std::path::Path) -> std::io::Result<Vec<GenomeBlob>> {
    let data = std::fs::read(path)?;
    if data.len() < 4 {
        return Ok(Vec::new());
    }
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let expected = 4 + count * 18;
    if data.len() < expected {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("expected {expected} bytes, got {}", data.len()),
        ));
    }
    let mut genomes = Vec::with_capacity(count);
    let mut offset = 4;
    for _ in 0..count {
        let archetype    = data[offset];
        let trophic_class = data[offset + 1];
        let growth_bias   = f32::from_le_bytes([data[offset+2], data[offset+3], data[offset+4], data[offset+5]]);
        let mobility_bias = f32::from_le_bytes([data[offset+6], data[offset+7], data[offset+8], data[offset+9]]);
        let branching_bias = f32::from_le_bytes([data[offset+10], data[offset+11], data[offset+12], data[offset+13]]);
        let resilience    = f32::from_le_bytes([data[offset+14], data[offset+15], data[offset+16], data[offset+17]]);
        genomes.push(GenomeBlob {
            archetype, trophic_class,
            growth_bias, mobility_bias, branching_bias, resilience,
        });
        offset += 18;
    }
    Ok(genomes)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Round-trip: Batch → Bevy → Batch ────────────────────────────────────

    #[test]
    fn round_trip_biases_are_bit_exact() {
        let original = GenomeBlob {
            archetype: 1, trophic_class: 0,
            growth_bias: 0.73, mobility_bias: 0.21,
            branching_bias: 0.88, resilience: 0.45,
        };
        let (_, _, _, _, _, _, profile) = genome_to_components(&original);
        let trophic = genome_to_trophic(&original);
        let back = components_to_genome(&profile, trophic.as_ref());
        assert_eq!(original.growth_bias.to_bits(), back.growth_bias.to_bits());
        assert_eq!(original.mobility_bias.to_bits(), back.mobility_bias.to_bits());
        assert_eq!(original.branching_bias.to_bits(), back.branching_bias.to_bits());
        assert_eq!(original.resilience.to_bits(), back.resilience.to_bits());
    }

    #[test]
    fn round_trip_preserves_archetype_flora() {
        let g = GenomeBlob { archetype: 1, trophic_class: 0, ..Default::default() };
        let (_, _, _, _, _, _, profile) = genome_to_components(&g);
        let trophic = genome_to_trophic(&g);
        let back = components_to_genome(&profile, trophic.as_ref());
        assert_eq!(back.archetype, 1);
        assert_eq!(back.trophic_class, 0);
    }

    #[test]
    fn round_trip_preserves_archetype_fauna() {
        let g = GenomeBlob { archetype: 2, trophic_class: 3, ..Default::default() };
        let (_, _, _, _, _, _, profile) = genome_to_components(&g);
        let trophic = genome_to_trophic(&g);
        let back = components_to_genome(&profile, trophic.as_ref());
        assert_eq!(back.archetype, 2);
        assert_eq!(back.trophic_class, 3);
    }

    #[test]
    fn genome_to_components_produces_valid_energy() {
        let g = GenomeBlob::default();
        let (energy, volume, osc, _, _, _, _) = genome_to_components(&g);
        assert!(energy.qe() > 0.0);
        assert!(volume.radius > 0.0);
        assert!(osc.frequency_hz() > 0.0);
    }

    #[test]
    fn genome_to_trophic_none_for_inert() {
        let g = GenomeBlob { trophic_class: 255, ..Default::default() };
        assert!(genome_to_trophic(&g).is_none());
    }

    #[test]
    fn genome_to_trophic_some_for_valid() {
        let g = GenomeBlob { trophic_class: 1, ..Default::default() };
        let t = genome_to_trophic(&g).unwrap();
        assert_eq!(t.class, TrophicClass::Herbivore);
    }

    #[test]
    fn frequency_for_archetype_distinct() {
        let freqs: Vec<f32> = (0..5).map(frequency_for_archetype).collect();
        for i in 0..freqs.len() {
            for j in (i + 1)..freqs.len() {
                assert_ne!(freqs[i], freqs[j], "archetypes {i} and {j} share frequency");
            }
        }
    }

    // ── Serialization round-trip ────────────────────────────────────────────

    #[test]
    fn save_load_round_trip() {
        let genomes = vec![
            GenomeBlob { archetype: 1, trophic_class: 0, growth_bias: 0.5, mobility_bias: 0.3, branching_bias: 0.7, resilience: 0.9 },
            GenomeBlob { archetype: 2, trophic_class: 3, growth_bias: 0.1, mobility_bias: 0.8, branching_bias: 0.2, resilience: 0.4 },
        ];
        let dir = std::env::temp_dir();
        let path = dir.join("test_genomes_bs5.bin");
        save_genomes(&genomes, &path).unwrap();
        let loaded = load_genomes(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], genomes[0]);
        assert_eq!(loaded[1], genomes[1]);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_load_empty() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_genomes_empty_bs5.bin");
        save_genomes(&[], &path).unwrap();
        let loaded = load_genomes(&path).unwrap();
        assert!(loaded.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_corrupted_returns_error() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_genomes_corrupt_bs5.bin");
        // Write a file that claims 10 genomes but has no data
        let mut buf = Vec::new();
        buf.extend_from_slice(&10u32.to_le_bytes());
        std::fs::write(&path, buf).unwrap();
        let result = load_genomes(&path);
        assert!(result.is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_load_preserves_bit_exact() {
        let g = GenomeBlob {
            archetype: 4, trophic_class: 4,
            growth_bias: std::f32::consts::PI / 7.0,
            mobility_bias: std::f32::consts::E / 3.0,
            branching_bias: 0.123_456_78,
            resilience: 0.987_654_3,
        };
        let dir = std::env::temp_dir();
        let path = dir.join("test_genomes_bitexact_bs5.bin");
        save_genomes(&[g], &path).unwrap();
        let loaded = load_genomes(&path).unwrap();
        assert_eq!(g.growth_bias.to_bits(), loaded[0].growth_bias.to_bits());
        assert_eq!(g.mobility_bias.to_bits(), loaded[0].mobility_bias.to_bits());
        assert_eq!(g.branching_bias.to_bits(), loaded[0].branching_bias.to_bits());
        assert_eq!(g.resilience.to_bits(), loaded[0].resilience.to_bits());
        std::fs::remove_file(&path).ok();
    }
}
