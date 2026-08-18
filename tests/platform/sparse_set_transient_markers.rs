//! Contrato de regresión: marcadores transitorios del gameplay deben usar `StorageType::SparseSet`.
//! Si se revierte `#[component(storage = "SparseSet")]` sin decisión explícita, CI falla.
//! Patrón G1 (SparseSet) — `docs/sprints/GAMEDEV_PATTERNS/README.md` (sprint doc eliminado).

use bevy::ecs::component::{Component, StorageType};
use resonance::layers::{DespawnOnContact, OnContactEffect};
use resonance::simulation::{PlayerControlled, SpellMarker};

#[test]
fn despawn_on_contact_is_sparse_set() {
    assert_eq!(DespawnOnContact::STORAGE_TYPE, StorageType::SparseSet);
}

#[test]
fn on_contact_effect_is_sparse_set() {
    assert_eq!(OnContactEffect::STORAGE_TYPE, StorageType::SparseSet);
}

#[test]
fn spell_marker_is_sparse_set() {
    assert_eq!(SpellMarker::STORAGE_TYPE, StorageType::SparseSet);
}

#[test]
fn player_controlled_is_sparse_set() {
    assert_eq!(PlayerControlled::STORAGE_TYPE, StorageType::SparseSet);
}
