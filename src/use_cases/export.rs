//! Export adapters — transforman datos de simulación a formatos científicos.
//! Export adapters — transform simulation data to scientific formats.
//!
//! Stateless: cada función recibe data, retorna String. Zero IO.
//! El caller decide dónde persistir (archivo, stdout, red).

use std::fmt::Write as _;

use crate::batch::census::{EntitySnapshot, PopulationCensus};
use crate::batch::harness::GenerationStats;

// ─── CSV ────────────────────────────────────────────────────────────────────

/// Header CSV para snapshots de entidad (sin prefijo de generación).
/// CSV header for entity snapshots (no generation prefix).
pub const ENTITY_CSV_HEADER: &str =
    "world,slot,archetype,alive,qe,radius,freq_hz,growth,mobility,branching,resilience,trophic,age,lineage_id";

/// Header CSV para stats de generación.
/// CSV header for generation stats.
pub const GENERATION_CSV_HEADER: &str =
    "gen,best,mean,worst,diversity,survivors,species,genes,metab_rate,protein_rate,codons,multicell_rate";

/// Escribe un EntitySnapshot como línea CSV en el buffer.
/// Writes an EntitySnapshot as a CSV line into the buffer.
#[inline]
pub fn write_entity_csv(out: &mut String, s: &EntitySnapshot) {
    let _ = write!(out,
        "{},{},{},{},{:.4},{:.4},{:.2},{:.4},{:.4},{:.4},{:.4},{},{},{}",
        s.world_index, s.slot_index, s.archetype,
        s.alive as u8, s.qe, s.radius, s.frequency_hz,
        s.growth_bias, s.mobility_bias, s.branching_bias, s.resilience,
        s.trophic_class, s.age_ticks, s.lineage_id.0,
    );
}

/// Escribe GenerationStats como línea CSV en el buffer.
/// Writes GenerationStats as a CSV line into the buffer.
#[inline]
pub fn write_generation_csv(out: &mut String, s: &GenerationStats) {
    let _ = write!(out,
        "{},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{:.2},{:.4},{:.4},{:.2},{:.4}",
        s.generation,
        s.best_fitness, s.mean_fitness, s.worst_fitness,
        s.diversity, s.survivors_mean, s.species_mean,
        s.gene_count_mean, s.metabolic_graph_rate,
        s.protein_function_rate, s.codon_count_mean,
        s.multicellular_rate,
    );
}

/// Convenience: serializa un EntitySnapshot como String CSV (para callers puntuales).
/// Convenience: serializes an EntitySnapshot as a CSV String (for one-off callers).
#[inline]
pub fn entity_to_csv(s: &EntitySnapshot) -> String {
    let mut out = String::with_capacity(120);
    write_entity_csv(&mut out, s);
    out
}

/// Convenience: serializa GenerationStats como String CSV.
/// Convenience: serializes GenerationStats as a CSV String.
#[inline]
pub fn generation_to_csv(s: &GenerationStats) -> String {
    let mut out = String::with_capacity(120);
    write_generation_csv(&mut out, s);
    out
}

// ─── Bulk export (HOFs) ─────────────────────────────────────────────────────

/// Exporta historia completa de generaciones como CSV.
/// Exports complete generation history as CSV.
pub fn export_history_csv(history: &[GenerationStats]) -> String {
    let mut out = String::with_capacity(history.len() * 120 + 200);
    out.push_str(GENERATION_CSV_HEADER);
    out.push('\n');
    for stats in history {
        write_generation_csv(&mut out, stats);
        out.push('\n');
    }
    out
}

/// Exporta todos los censos como CSV (prefija generation number).
/// Exports all censuses as CSV (prefixes generation number).
pub fn export_censuses_csv(censuses: &[PopulationCensus]) -> String {
    let entity_count: usize = censuses.iter().map(|c| c.snapshots.len()).sum();
    let mut out = String::with_capacity(entity_count * 100 + 200);
    out.push_str("gen,");
    out.push_str(ENTITY_CSV_HEADER);
    out.push('\n');
    for census in censuses {
        for snap in census.alive() {
            let _ = write!(out, "{},", census.generation);
            write_entity_csv(&mut out, snap);
            out.push('\n');
        }
    }
    out
}

// ─── JSON ───────────────────────────────────────────────────────────────────

/// Serializa un EntitySnapshot como JSON object (sin dependencia de serde_json).
/// Serializes an EntitySnapshot as a JSON object (no serde_json dependency).
#[inline]
pub fn entity_to_json(s: &EntitySnapshot) -> String {
    format!(
        r#"{{"lineage":{},"world":{},"slot":{},"arch":{},"alive":{},"qe":{:.4},"r":{:.4},"hz":{:.2},"g":{:.4},"m":{:.4},"b":{:.4},"res":{:.4},"troph":{},"age":{}}}"#,
        s.lineage_id.0, s.world_index, s.slot_index, s.archetype,
        s.alive, s.qe, s.radius, s.frequency_hz,
        s.growth_bias, s.mobility_bias, s.branching_bias, s.resilience,
        s.trophic_class, s.age_ticks,
    )
}

/// Serializa GenerationStats como JSON object.
/// Serializes GenerationStats as a JSON object.
#[inline]
pub fn generation_to_json(s: &GenerationStats) -> String {
    format!(
        r#"{{"gen":{},"best":{:.4},"mean":{:.4},"worst":{:.4},"div":{:.4},"surv":{:.2},"spp":{:.2},"genes":{:.2},"metab":{:.4},"protein":{:.4},"codons":{:.2},"multicell":{:.4}}}"#,
        s.generation, s.best_fitness, s.mean_fitness,
        s.worst_fitness, s.diversity, s.survivors_mean, s.species_mean,
        s.gene_count_mean, s.metabolic_graph_rate,
        s.protein_function_rate, s.codon_count_mean,
        s.multicellular_rate,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::lineage::LineageId;

    fn sample_snapshot() -> EntitySnapshot {
        EntitySnapshot {
            lineage_id:     LineageId::root(42, 0),
            world_index:    0,
            slot_index:     3,
            archetype:      2,
            alive:          true,
            qe:             150.5,
            radius:         2.0,
            frequency_hz:   97.5,
            growth_bias:    0.6,
            mobility_bias:  0.3,
            branching_bias: 0.1,
            resilience:     0.4,
            trophic_class:  1,
            age_ticks:      42,
        }
    }

    fn sample_stats() -> GenerationStats {
        GenerationStats {
            generation:           10,
            best_fitness:         85.0,
            mean_fitness:         42.0,
            worst_fitness:        1.0,
            diversity:            0.35,
            survivors_mean:       4.5,
            species_mean:         2.1,
            gene_count_mean:      8.0,
            metabolic_graph_rate: 0.25,
            protein_function_rate:0.10,
            codon_count_mean:     3.5,
            multicellular_rate:   0.05,
        }
    }

    // ── CSV ──

    #[test]
    fn entity_csv_has_correct_field_count() {
        let csv = entity_to_csv(&sample_snapshot());
        let fields: Vec<&str> = csv.split(',').collect();
        let header_fields: Vec<&str> = ENTITY_CSV_HEADER.split(',').collect();
        assert_eq!(fields.len(), header_fields.len(),
            "entity CSV field count must match header");
    }

    #[test]
    fn generation_csv_has_correct_field_count() {
        let csv = generation_to_csv(&sample_stats());
        let fields: Vec<&str> = csv.split(',').collect();
        let header_fields: Vec<&str> = GENERATION_CSV_HEADER.split(',').collect();
        assert_eq!(fields.len(), header_fields.len());
    }

    #[test]
    fn export_history_csv_starts_with_header() {
        let history = vec![sample_stats()];
        let csv = export_history_csv(&history);
        assert!(csv.starts_with(GENERATION_CSV_HEADER));
    }

    #[test]
    fn export_history_csv_one_line_per_generation() {
        let history = vec![sample_stats(), sample_stats()];
        let csv = export_history_csv(&history);
        let lines: Vec<&str> = csv.trim().lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 data lines");
    }

    #[test]
    fn export_censuses_csv_prefixes_generation() {
        let census = PopulationCensus {
            generation: 7,
            snapshots: vec![sample_snapshot()],
        };
        let csv = export_censuses_csv(&[census]);
        let data_line = csv.lines().nth(1).expect("should have data line");
        assert!(data_line.starts_with("7,"), "data line should start with generation number");
    }

    #[test]
    fn export_censuses_csv_empty_returns_header_only() {
        let csv = export_censuses_csv(&[]);
        let lines: Vec<&str> = csv.trim().lines().collect();
        assert_eq!(lines.len(), 1, "only header line");
    }

    // ── JSON ──

    #[test]
    fn entity_json_contains_lineage() {
        let json = entity_to_json(&sample_snapshot());
        assert!(json.contains("\"lineage\":"));
    }

    #[test]
    fn entity_json_contains_all_fields() {
        let json = entity_to_json(&sample_snapshot());
        for field in ["lineage", "world", "slot", "arch", "alive", "qe", "r", "hz", "g", "m", "b", "res", "troph", "age"] {
            assert!(json.contains(&format!("\"{}\":", field)), "missing field: {field}");
        }
    }

    #[test]
    fn generation_json_contains_gen_number() {
        let json = generation_to_json(&sample_stats());
        assert!(json.contains("\"gen\":10"));
    }
}
