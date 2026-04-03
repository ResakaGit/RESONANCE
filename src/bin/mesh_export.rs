//! C4: Mesh Export — export evolved creatures as OBJ for Blender / 3D printing.
//!
//! Usage:
//!   cargo run --release --bin mesh_export -- --genomes assets/evolved/seed_42.bin --out creatures/
//!   cargo run --release --bin mesh_export -- --seed 42 --gens 100 --out creatures/

use resonance::use_cases::cli::{archetype_label, find_arg, parse_arg};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_dir = find_arg(&args, "--out").unwrap_or_else(|| "exported_meshes".to_string());
    std::fs::create_dir_all(&out_dir).ok();

    let genomes = if let Some(path) = find_arg(&args, "--genomes") {
        println!("\n  Loading genomes from {path}...");
        resonance::batch::bridge::load_genomes(Path::new(&path)).unwrap_or_else(|e| {
            eprintln!("  Error: {e}");
            Vec::new()
        })
    } else {
        let seed = parse_arg(&args, "--seed", 42);
        let gens = parse_arg(&args, "--gens", 100);
        let ticks = parse_arg(&args, "--ticks", 500);
        println!("\n  Evolving genomes (seed={seed}, gens={gens})...");
        let report = resonance::use_cases::evolve_with(
            &resonance::use_cases::presets::EARTH,
            seed as u64,
            200,
            gens as u32,
            ticks as u32,
            12,
        );
        report.top_genomes
    };

    if genomes.is_empty() {
        println!("  No genomes to export.\n");
        return;
    }

    println!("  Exporting {} genomes to {out_dir}/...\n", genomes.len());

    for (i, genome) in genomes.iter().enumerate() {
        let obj = resonance::use_cases::experiments::mesh_export::genome_to_obj(genome);
        let label = archetype_label(genome.archetype);
        let filename = format!("{out_dir}/creature_{i:02}_{label}.obj");
        match std::fs::write(&filename, &obj) {
            Ok(()) => {
                let verts = obj.lines().filter(|l| l.starts_with("v ")).count();
                let faces = obj.lines().filter(|l| l.starts_with("f ")).count();
                println!("  {filename}: {verts} vertices, {faces} faces");
            }
            Err(e) => println!("  Error writing {filename}: {e}"),
        }
    }

    println!("\n  Done. Import OBJ files into Blender / slicer for 3D printing.\n");
}
