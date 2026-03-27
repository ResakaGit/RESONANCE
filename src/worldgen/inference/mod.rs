pub mod organ;
pub mod shape;
pub mod visual_derivation;

pub use organ::{AttachmentZone, OrganAttachment, build_organ_mesh, organ_attachment_points, organ_orientation};
pub use shape::{
    GeometryInferenceInput, GrowthMorphParams, PendingGrowthMorphRebuild,
    ShapeInferenceFrameState, ShapeInferenceParams, ShapeInferred,
    derive_geometry_influence, growth_morphology_system,
    reset_shape_inference_frame_system, shape_color_inference_system,
};
pub use visual_derivation::{
    VisualProperties, apply_archetype_visual_profile, boundary_transition_emission_extra,
    color_lerp, compound_color_blend, derive_all, derive_color, derive_color_compound,
    derive_color_phenology, derive_emission, derive_opacity, derive_scale,
    energy_visual_boundary_flat_color, materialized_tile_spatial_density,
    neutral_visual_linear_rgb, visual_proxy_temperature, zone_class_display_color,
};
