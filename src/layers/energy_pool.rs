use bevy::prelude::*;

use crate::blueprint::constants::{
    DISSIPATION_RATE_DEFAULT, DISSIPATION_RATE_MAX, DISSIPATION_RATE_MIN, POOL_CAPACITY_MIN,
};

/// Pool de energía distribuible a entidades hijas.
/// Invariante: Σ extracted(children) ≤ pool por tick.
/// Ortogonal a BaseEnergy: una entidad puede tener ambos.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct EnergyPool {
    pool: f32,
    capacity: f32,
    intake_rate: f32,
    dissipation_rate: f32,
}

impl Default for EnergyPool {
    fn default() -> Self {
        Self::new(0.0, POOL_CAPACITY_MIN, 0.0, DISSIPATION_RATE_DEFAULT)
    }
}

impl EnergyPool {
    pub fn new(pool: f32, capacity: f32, intake_rate: f32, dissipation_rate: f32) -> Self {
        let cap = capacity.max(POOL_CAPACITY_MIN);
        Self {
            pool: pool.max(0.0).min(cap),
            capacity: cap,
            intake_rate: intake_rate.max(0.0),
            dissipation_rate: dissipation_rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX),
        }
    }

    #[inline]
    pub fn pool(&self) -> f32 {
        self.pool
    }
    #[inline]
    pub fn capacity(&self) -> f32 {
        self.capacity
    }
    #[inline]
    pub fn intake_rate(&self) -> f32 {
        self.intake_rate
    }
    #[inline]
    pub fn dissipation_rate(&self) -> f32 {
        self.dissipation_rate
    }

    /// pool / capacity — derivado, no almacenado.
    #[inline]
    pub fn pool_ratio(&self) -> f32 {
        self.pool / self.capacity
    }

    pub fn set_pool(&mut self, val: f32) {
        self.pool = val.max(0.0).min(self.capacity);
    }

    pub fn set_capacity(&mut self, val: f32) {
        self.capacity = val.max(POOL_CAPACITY_MIN);
        if self.pool > self.capacity {
            self.pool = self.capacity;
        }
    }

    pub fn set_intake_rate(&mut self, val: f32) {
        self.intake_rate = val.max(0.0);
    }

    /// Reduce capacity (Type IV degradación estructural). Clamps pool si excede.
    pub fn degrade_capacity(&mut self, amount: f32) {
        self.capacity = (self.capacity - amount.max(0.0)).max(POOL_CAPACITY_MIN);
        if self.pool > self.capacity {
            self.pool = self.capacity;
        }
    }

    /// Añade energía al pool, clamped a capacity.
    pub fn replenish(&mut self, amount: f32) {
        self.pool = (self.pool + amount.max(0.0)).min(self.capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fields_correct() {
        let p = EnergyPool::new(500.0, 1000.0, 50.0, 0.01);
        assert_eq!(p.pool(), 500.0);
        assert_eq!(p.capacity(), 1000.0);
        assert_eq!(p.intake_rate(), 50.0);
        assert_eq!(p.dissipation_rate(), 0.01);
    }

    #[test]
    fn pool_clamped_to_capacity() {
        let p = EnergyPool::new(2000.0, 1000.0, 0.0, 0.01);
        assert_eq!(p.pool(), 1000.0);
    }

    #[test]
    fn pool_clamped_to_zero() {
        let p = EnergyPool::new(-50.0, 1000.0, 0.0, 0.01);
        assert_eq!(p.pool(), 0.0);
    }

    #[test]
    fn capacity_clamped_to_min() {
        let p = EnergyPool::new(0.0, 0.0, 0.0, 0.01);
        assert_eq!(p.capacity(), POOL_CAPACITY_MIN);
    }

    #[test]
    fn dissipation_rate_clamped_low() {
        let p = EnergyPool::new(100.0, 1000.0, 0.0, 0.0);
        assert_eq!(p.dissipation_rate(), DISSIPATION_RATE_MIN);
    }

    #[test]
    fn dissipation_rate_clamped_high() {
        let p = EnergyPool::new(100.0, 1000.0, 0.0, 1.0);
        assert_eq!(p.dissipation_rate(), DISSIPATION_RATE_MAX);
    }

    #[test]
    fn intake_rate_clamped_non_negative() {
        let p = EnergyPool::new(100.0, 1000.0, -10.0, 0.01);
        assert_eq!(p.intake_rate(), 0.0);
    }

    #[test]
    fn pool_ratio_normal() {
        let p = EnergyPool::new(500.0, 1000.0, 0.0, 0.01);
        assert!((p.pool_ratio() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn pool_ratio_zero_capacity_no_nan() {
        // capacity clamped to POOL_CAPACITY_MIN, so no division by zero
        let p = EnergyPool::new(0.0, 0.0, 0.0, 0.01);
        assert!(!p.pool_ratio().is_nan());
        assert_eq!(p.pool_ratio(), 0.0);
    }

    #[test]
    fn degrade_capacity_reduces_and_clamps_pool() {
        let mut p = EnergyPool::new(900.0, 1000.0, 0.0, 0.01);
        p.degrade_capacity(100.0);
        assert_eq!(p.capacity(), 900.0);
        assert_eq!(p.pool(), 900.0);
    }

    #[test]
    fn degrade_capacity_clamps_to_min() {
        let mut p = EnergyPool::new(500.0, 1000.0, 0.0, 0.01);
        p.degrade_capacity(5000.0);
        assert_eq!(p.capacity(), POOL_CAPACITY_MIN);
        assert_eq!(p.pool(), POOL_CAPACITY_MIN);
    }

    #[test]
    fn replenish_adds_clamped_to_capacity() {
        let mut p = EnergyPool::new(500.0, 1000.0, 0.0, 0.01);
        p.replenish(300.0);
        assert_eq!(p.pool(), 800.0);
        p.replenish(500.0);
        assert_eq!(p.pool(), 1000.0);
    }

    #[test]
    fn replenish_negative_does_nothing() {
        let mut p = EnergyPool::new(500.0, 1000.0, 0.0, 0.01);
        p.replenish(-100.0);
        assert_eq!(p.pool(), 500.0);
    }

    #[test]
    fn set_pool_clamps() {
        let mut p = EnergyPool::new(500.0, 1000.0, 0.0, 0.01);
        p.set_pool(2000.0);
        assert_eq!(p.pool(), 1000.0);
        p.set_pool(-10.0);
        assert_eq!(p.pool(), 0.0);
    }

    #[test]
    fn set_capacity_clamps_pool_if_exceeds() {
        let mut p = EnergyPool::new(800.0, 1000.0, 0.0, 0.01);
        p.set_capacity(500.0);
        assert_eq!(p.capacity(), 500.0);
        assert_eq!(p.pool(), 500.0);
    }

    #[test]
    fn set_intake_rate_clamps() {
        let mut p = EnergyPool::new(100.0, 1000.0, 50.0, 0.01);
        p.set_intake_rate(-5.0);
        assert_eq!(p.intake_rate(), 0.0);
        p.set_intake_rate(100.0);
        assert_eq!(p.intake_rate(), 100.0);
    }

    #[test]
    fn size_of_energy_pool_is_16_bytes() {
        assert_eq!(std::mem::size_of::<EnergyPool>(), 4 * 4);
    }

    #[test]
    fn energy_pool_is_copy() {
        let a = EnergyPool::new(100.0, 200.0, 10.0, 0.01);
        let b = a;
        assert_eq!(a, b);
    }
}
