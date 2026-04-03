//! GF2C — Caché de geometría deformada.
//!
//! Parallel-array al grid: cada entrada guarda el último resultado deformado
//! y el rango de tensor válido. Hit cuando fingerprint coincide Y tensor ∈ [min, max].

use bevy::prelude::Resource;

struct CacheEntry {
    fingerprint: u64,
    deformed_positions: Vec<[f32; 3]>,
    tensor_min: f32,
    tensor_max: f32,
    range_width_factor: f32,
    hits: u32,
    misses: u32,
}

impl CacheEntry {
    fn empty() -> Self {
        Self {
            fingerprint: 0,
            deformed_positions: Vec::new(),
            tensor_min: 0.0,
            tensor_max: 0.0,
            range_width_factor: 1.0,
            hits: 0,
            misses: 0,
        }
    }
}

/// Caché de deformación geométrica (parallel-array al grid de entidades).
#[derive(Resource)]
pub struct GeometryDeformationCache {
    entries: Vec<CacheEntry>,
}

impl Default for GeometryDeformationCache {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl GeometryDeformationCache {
    pub fn new(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(CacheEntry::empty());
        }
        Self { entries }
    }

    /// Consulta el cache para el slot `idx`.
    ///
    /// Hit: `fingerprint` coincide Y `tensor_magnitude` ∈ `[tensor_min, tensor_max]`.
    pub fn lookup(
        &self,
        fingerprint: u64,
        tensor_magnitude: f32,
        idx: usize,
    ) -> Option<&Vec<[f32; 3]>> {
        let entry = self.entries.get(idx)?;
        if entry.fingerprint == fingerprint
            && tensor_magnitude >= entry.tensor_min
            && tensor_magnitude <= entry.tensor_max
        {
            Some(&entry.deformed_positions)
        } else {
            None
        }
    }

    /// Actualiza el slot `idx` con nuevas posiciones deformadas y desliza el rango de tensor.
    pub fn update(
        &mut self,
        idx: usize,
        fingerprint: u64,
        positions: Vec<[f32; 3]>,
        tensor_magnitude: f32,
    ) {
        // Crecer si es necesario.
        if idx >= self.entries.len() {
            self.entries.resize_with(idx + 1, CacheEntry::empty);
        }
        let entry = &mut self.entries[idx];

        let half_width = entry.range_width_factor * 0.5;
        let new_min = tensor_magnitude - half_width;
        let new_max = tensor_magnitude + half_width;

        // Deslizar el rango hacia el nuevo centro; reducir levemente el factor.
        if entry.fingerprint == fingerprint {
            // Miss en rango: ajustar hacia nuevo valor, reducir factor.
            entry.tensor_min = new_min;
            entry.tensor_max = new_max;
            entry.range_width_factor *= 0.9999;
        } else {
            // Miss total: nuevo fingerprint → reset completo del rango.
            entry.tensor_min = new_min;
            entry.tensor_max = new_max;
            entry.range_width_factor = 1.0;
        }

        entry.fingerprint = fingerprint;
        entry.deformed_positions = positions;
        entry.misses += 1;
    }

    /// Incrementa el contador de hits del slot (llamado cuando `lookup` devuelve `Some`).
    pub fn record_hit(&mut self, idx: usize) {
        if let Some(entry) = self.entries.get_mut(idx) {
            entry.hits += 1;
        }
    }

    /// Estadísticas acumuladas de todos los slots.
    pub fn total_hits(&self) -> u32 {
        self.entries.iter().map(|e| e.hits).sum()
    }

    pub fn total_misses(&self) -> u32 {
        self.entries.iter().map(|e| e.misses).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
    }

    #[test]
    fn second_call_same_payload_returns_hit() {
        let mut cache = GeometryDeformationCache::new(4);
        let fp = 0xDEADBEEF_u64;
        let tensor = 5.0_f32;

        // Primera llamada: miss, store.
        assert!(cache.lookup(fp, tensor, 0).is_none());
        cache.update(0, fp, sample_positions(), tensor);

        // Segunda llamada: mismo fingerprint y tensor en rango → hit.
        let result = cache.lookup(fp, tensor, 0);
        assert!(result.is_some(), "second call must be a cache hit");
        cache.record_hit(0);
        assert_eq!(cache.total_hits(), 1);
    }

    #[test]
    fn tensor_out_of_range_is_miss_and_range_slides() {
        let mut cache = GeometryDeformationCache::new(4);
        let fp = 0xCAFEBABE_u64;
        let tensor_a = 5.0_f32;
        let tensor_b = 100.0_f32; // fuera de rango

        cache.update(0, fp, sample_positions(), tensor_a);
        let range_width_before = cache.entries[0].range_width_factor;

        // Tensor fuera del rango actual → miss.
        assert!(
            cache.lookup(fp, tensor_b, 0).is_none(),
            "tensor out of range must miss"
        );

        // Actualizar con el nuevo tensor → desliza rango.
        cache.update(0, fp, sample_positions(), tensor_b);
        let range_width_after = cache.entries[0].range_width_factor;

        // El rango se deslizó hacia tensor_b.
        let mid = (cache.entries[0].tensor_min + cache.entries[0].tensor_max) / 2.0;
        assert!(
            (mid - tensor_b).abs() < 0.5,
            "range center must slide toward new tensor: {mid} vs {tensor_b}"
        );
        // El factor se redujo por el miss de mismo fingerprint.
        assert!(
            range_width_after <= range_width_before,
            "range_width_factor must shrink after miss: {range_width_after} vs {range_width_before}"
        );
    }

    #[test]
    fn different_fingerprint_total_reset() {
        let mut cache = GeometryDeformationCache::new(2);
        let fp_a = 0xAAAA_u64;
        let fp_b = 0xBBBB_u64;

        cache.update(0, fp_a, sample_positions(), 1.0);
        cache.update(0, fp_b, sample_positions(), 1.0);

        // El nuevo fingerprint debe ser un miss para fp_a.
        assert!(cache.lookup(fp_a, 1.0, 0).is_none());
        // Y un hit para fp_b.
        assert!(cache.lookup(fp_b, 1.0, 0).is_some());
    }

    #[test]
    fn lookup_out_of_bounds_returns_none() {
        let cache = GeometryDeformationCache::new(2);
        assert!(cache.lookup(0, 0.0, 99).is_none());
    }

    #[test]
    fn total_hits_and_misses_accumulate() {
        let mut cache = GeometryDeformationCache::new(2);
        let fp = 42_u64;
        // Miss inicial (vacío).
        assert!(cache.lookup(fp, 1.0, 0).is_none());
        cache.update(0, fp, sample_positions(), 1.0);

        // Hit.
        cache.lookup(fp, 1.0, 0);
        cache.record_hit(0);
        cache.lookup(fp, 1.0, 0);
        cache.record_hit(0);

        assert_eq!(cache.total_hits(), 2);
        assert_eq!(cache.total_misses(), 1); // sólo el update cuenta como miss
    }
}
