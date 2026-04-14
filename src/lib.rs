// ─── Clippy configuration ───────────────────────────────────────────────────
// Bevy ECS systems require many query parameters and produce complex types.
// These are false positives in the Bevy ecosystem — every Bevy project allows them.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
// EntitySlot field-by-field init is the canonical batch pattern (repr(C) struct).
#![allow(clippy::field_reassign_with_default)]
// Index loops are idiomatic for fixed-size array math (radial fields, gene arrays).
// Iterator+enumerate versions are less readable for 2D array indexing.
#![allow(clippy::needless_range_loop)]
// Bilingual doc comments (Spanish/English) sometimes have structure that triggers this.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::empty_line_after_doc_comments)]
// let-else vs ? is a style choice; let-else is more explicit in systems.
#![allow(clippy::question_mark)]
// Collapsible match patterns reduce readability in Bevy mesh attribute handling.
#![allow(clippy::collapsible_match)]
// Manual clamp is clearer than .clamp() when min/max are asymmetric expressions.
#![allow(clippy::manual_clamp)]
// Slice::from_ref is less idiomatic than &[x.clone()] in most contexts.
#![allow(clippy::cloned_ref_to_slice_refs)]
// Remaining minor style lints (8 individual occurrences across 113K LOC).
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::module_inception)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::manual_memcpy)]
#![allow(clippy::disallowed_names)]
#![allow(clippy::explicit_counter_loop)]

pub mod batch;
pub mod blueprint;
pub mod bridge;
pub mod cosmic;
pub mod eco;
pub mod events;
pub mod geometry_flow;
pub mod math_types;
pub mod plugins;
pub mod rendering;
pub mod runtime_platform;
pub mod sim_world;

pub mod entities;
pub mod layers;
pub mod simulation;
pub mod topology;
pub mod use_cases;
pub mod viewer;
pub mod world;
pub mod worldgen;
