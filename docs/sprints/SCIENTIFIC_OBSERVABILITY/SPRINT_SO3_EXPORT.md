# SO-3: Export Pipeline (CSV + JSON Adapters)

**Objetivo:** Transformar `PopulationCensus` y `GenerationStats` en formatos consumibles por herramientas científicas (Python pandas, R, Excel, Jupyter). Stateless adapters — zero estado, zero IO interno (el caller decide dónde escribir).

**Estado:** PENDIENTE
**Esfuerzo:** M (~120 LOC)
**Bloqueado por:** SO-2

---

## Diseño: Trait `ExportAdapter`

```rust
// src/use_cases/export.rs (NUEVO)

/// Adapter stateless para serializar datos de simulación.
/// Stateless adapter for serializing simulation data.
///
/// Cada implementación transforma data → bytes. No hace IO — retorna String o Vec<u8>.
/// El caller (binary, test, API) decide dónde persistir.
pub trait ExportAdapter {
    /// Header row para formats tabulares (CSV). None para formatos jerárquicos.
    fn header() -> Option<String>;

    /// Serializa un snapshot de entidad.
    fn entity(snapshot: &EntitySnapshot) -> String;

    /// Serializa un resumen de generación.
    fn generation(stats: &GenerationStats) -> String;

    /// Serializa un census completo.
    fn census(census: &PopulationCensus) -> String {
        let mut out = String::new();
        if let Some(h) = Self::header() { out.push_str(&h); out.push('\n'); }
        for s in census.alive() {
            out.push_str(&Self::entity(s));
            out.push('\n');
        }
        out
    }
}
```

### CSV Adapter

```rust
pub struct CsvAdapter;

impl ExportAdapter for CsvAdapter {
    fn header() -> Option<String> {
        Some("gen,world,slot,archetype,qe,radius,freq_hz,growth,mobility,branching,resilience,trophic,age,lineage_id,alive".into())
    }

    fn entity(s: &EntitySnapshot) -> String {
        format!(
            "{},{},{},{},{:.4},{:.4},{:.2},{:.4},{:.4},{:.4},{:.4},{},{},{}",
            s.world_index, s.slot_index, s.archetype,
            s.alive as u8, s.qe, s.radius, s.frequency_hz,
            s.growth_bias, s.mobility_bias, s.branching_bias, s.resilience,
            s.trophic_class, s.age_ticks, s.lineage_id.0,
        )
    }

    fn generation(stats: &GenerationStats) -> String {
        format!(
            "{},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{:.2},{:.4},{:.4},{:.2},{:.4}",
            stats.generation,
            stats.best_fitness, stats.mean_fitness, stats.worst_fitness,
            stats.diversity, stats.survivors_mean, stats.species_mean,
            stats.gene_count_mean, stats.metabolic_graph_rate,
            stats.protein_function_rate, stats.codon_count_mean,
            stats.multicellular_rate,
        )
    }
}
```

### JSON Adapter

```rust
pub struct JsonAdapter;

impl ExportAdapter for JsonAdapter {
    fn header() -> Option<String> { None }

    fn entity(s: &EntitySnapshot) -> String {
        // Minimal JSON sin dependencia de serde_json (string formatting directo)
        format!(
            r#"{{"lineage":{},"world":{},"slot":{},"arch":{},"alive":{},"qe":{:.4},"r":{:.4},"hz":{:.2},"g":{:.4},"m":{:.4},"b":{:.4},"res":{:.4},"troph":{},"age":{}}}"#,
            s.lineage_id.0, s.world_index, s.slot_index, s.archetype,
            s.alive, s.qe, s.radius, s.frequency_hz,
            s.growth_bias, s.mobility_bias, s.branching_bias, s.resilience,
            s.trophic_class, s.age_ticks,
        )
    }

    fn generation(stats: &GenerationStats) -> String {
        format!(
            r#"{{"gen":{},"best":{:.4},"mean":{:.4},"worst":{:.4},"div":{:.4},"surv":{:.2},"spp":{:.2}}}"#,
            stats.generation, stats.best_fitness, stats.mean_fitness,
            stats.worst_fitness, stats.diversity, stats.survivors_mean, stats.species_mean,
        )
    }
}
```

### HOF: `export_history`

```rust
/// Exporta la historia completa de generaciones usando cualquier adapter.
/// Exports complete generation history using any adapter.
pub fn export_history<A: ExportAdapter>(history: &[GenerationStats]) -> String {
    let mut out = String::new();
    out.push_str("gen,best,mean,worst,diversity,survivors,species,genes,metab_rate,protein_rate,codons,multicell_rate\n");
    for stats in history {
        out.push_str(&A::generation(stats));
        out.push('\n');
    }
    out
}

/// Exporta todos los censos de una run.
pub fn export_all_censuses<A: ExportAdapter>(censuses: &[PopulationCensus]) -> String {
    let mut out = String::new();
    if let Some(h) = A::header() { out.push_str(&h); out.push('\n'); }
    for census in censuses {
        for s in census.alive() {
            out.push_str(&format!("{},", census.generation));
            out.push_str(&A::entity(s));
            out.push('\n');
        }
    }
    out
}
```

---

## Propiedad: Zero IO

Los adapters **nunca hacen IO**. Retornan `String`. El caller escribe a archivo:

```rust
// En el binary:
let csv = export_history::<CsvAdapter>(&harness.history);
std::fs::write("results.csv", csv)?;
```

Esto permite testear los adapters sin filesystem, usar en WASM, pipe a stdout, etc.

---

## Tests

```
// CsvAdapter
csv_header_has_correct_column_count
csv_entity_round_trips_alive_entity
csv_entity_round_trips_dead_entity
csv_generation_has_correct_field_count

// JsonAdapter
json_entity_is_valid_json
json_entity_contains_all_fields
json_generation_is_valid_json

// HOF
export_history_csv_starts_with_header
export_history_csv_has_one_line_per_generation
export_all_censuses_csv_prefixes_generation_number
export_all_censuses_empty_input_returns_header_only

// Trait contract
csv_census_uses_header_plus_entity_lines
json_census_has_no_header
```

---

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/use_cases/export.rs` | **NUEVO** — ExportAdapter trait + CSV + JSON |
| `src/use_cases/mod.rs` | + `pub mod export` |
