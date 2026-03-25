use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::*;
use crate::layers::{LifecycleStage, OrganManifest, OrganRole, OrganSpec};

#[inline]
fn count_to_u8_floor(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    value.floor().clamp(0.0, MAX_ORGAN_INSTANCE_COUNT as f32) as u8
}

#[inline]
fn nearest_fibonacci(value: u8) -> u8 {
    // Serie corta y estable para conteos orgánicos en runtime.
    const FIB: [u8; 7] = [0, 1, 2, 3, 5, 8, 13];
    let max = MAX_ORGAN_INSTANCE_COUNT;
    let target = value.min(max);
    let mut prev = 0u8;
    for f in FIB {
        if f > max {
            break;
        }
        if f >= target {
            let down = target.saturating_sub(prev);
            let up = f.saturating_sub(target);
            // En empate, preferimos el mayor para filotaxia más expresiva.
            return if up <= down { f } else { prev };
        }
        prev = f;
    }
    prev
}

/// Número de hojas inferido desde biomasa y sesgo de crecimiento.
#[inline]
pub fn infer_leaf_count(biomass: f32, growth_bias: f32) -> u8 {
    let bio = finite_non_negative(biomass);
    let growth = finite_unit(growth_bias);
    count_to_u8_floor(bio * growth * LEAF_COUNT_SCALE)
}

/// Número de pétalos inferido con preferencia por números de Fibonacci.
#[inline]
pub fn infer_petal_count(biomass: f32, branching_bias: f32) -> u8 {
    let bio = finite_non_negative(biomass);
    let branching = finite_unit(branching_bias);
    let raw = count_to_u8_floor(bio * branching * PETAL_COUNT_SCALE);
    nearest_fibonacci(raw)
}

/// Número de espinas inferido desde resiliencia.
#[inline]
pub fn infer_thorn_count(biomass: f32, resilience: f32) -> u8 {
    let bio = finite_non_negative(biomass);
    let robust = finite_unit(resilience);
    count_to_u8_floor(bio * robust * THORN_COUNT_SCALE)
}

/// Número de raíces inferido desde biomasa y sesgo anti-crecimiento.
#[inline]
pub fn infer_root_count(biomass: f32, growth_bias: f32) -> u8 {
    let bio = finite_non_negative(biomass);
    if bio <= 0.0 {
        return 0;
    }
    let growth = finite_unit(growth_bias);
    let raw = bio * (1.0 - growth) * ROOT_COUNT_SCALE + 1.0;
    count_to_u8_floor(raw).max(1)
}

/// Número de extremidades inferido; fuerza simetría bilateral (conteo par).
#[inline]
pub fn infer_limb_count(biomass: f32, mobility_bias: f32) -> u8 {
    let bio = finite_non_negative(biomass);
    let mobility = finite_unit(mobility_bias);
    let raw = count_to_u8_floor(bio * mobility * LIMB_COUNT_SCALE);
    (raw / 2 * 2).min(MAX_ORGAN_INSTANCE_COUNT)
}

#[inline]
fn push_if_nonzero(manifest: &mut OrganManifest, role: OrganRole, count: u8, scale_factor: f32) {
    if count == 0 {
        return;
    }
    let _ = manifest.push(OrganSpec::new(role, count, scale_factor));
}

#[derive(Clone, Copy)]
struct OrganInferenceCtx {
    caps: u8,
    growth: f32,
    mobility: f32,
    branching: f32,
    robust: f32,
    bio: f32,
    progress: f32,
    viability: f32,
}

#[inline]
fn has_capability(flags: u8, capability: u8) -> bool {
    flags & capability != 0
}

#[inline]
fn push_vegetative_roles(
    manifest: &mut OrganManifest,
    ctx: &OrganInferenceCtx,
    stem_scale: f32,
    leaf_count: u8,
    thorn_count: u8,
    include_move: bool,
) {
    push_if_nonzero(manifest, OrganRole::Stem, 1, stem_scale);
    if has_capability(ctx.caps, crate::layers::CapabilitySet::PHOTOSYNTH) {
        push_if_nonzero(manifest, OrganRole::Leaf, leaf_count, 0.5 * ctx.growth);
    }
    if has_capability(ctx.caps, crate::layers::CapabilitySet::ROOT) {
        push_if_nonzero(manifest, OrganRole::Root, infer_root_count(ctx.bio, ctx.growth), 0.4);
    }
    if has_capability(ctx.caps, crate::layers::CapabilitySet::BRANCH) {
        push_if_nonzero(manifest, OrganRole::Thorn, thorn_count, 0.2 * ctx.robust);
    }
    if has_capability(ctx.caps, crate::layers::CapabilitySet::SENSE) {
        push_if_nonzero(manifest, OrganRole::Sensory, 1, 0.15);
    }
    if has_capability(ctx.caps, crate::layers::CapabilitySet::ARMOR) {
        push_if_nonzero(manifest, OrganRole::Shell, 1, 0.3 * ctx.robust);
    }
    if include_move && has_capability(ctx.caps, crate::layers::CapabilitySet::MOVE) {
        let limbs = infer_limb_count(ctx.bio, ctx.mobility);
        push_if_nonzero(manifest, OrganRole::Limb, limbs, 0.6);
        push_if_nonzero(manifest, OrganRole::Fin, limbs, 0.4);
    }
}

#[inline]
fn push_reproductive_roles(manifest: &mut OrganManifest, ctx: &OrganInferenceCtx) {
    if !has_capability(ctx.caps, crate::layers::CapabilitySet::REPRODUCE) {
        return;
    }
    push_if_nonzero(
        manifest,
        OrganRole::Petal,
        infer_petal_count(ctx.bio, ctx.branching),
        0.5 * ctx.branching,
    );
    if ctx.bio > FRUIT_BIOMASS_THRESHOLD {
        push_if_nonzero(manifest, OrganRole::Fruit, 1, 0.4);
    }
}

/// Infiere el manifesto de órganos desde estado observable; función pura y stateless.
#[inline]
pub fn infer_organ_manifest(
    stage: LifecycleStage,
    capabilities: u8,
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
    biomass: f32,
    growth_progress: f32,
    viability: f32,
) -> OrganManifest {
    let ctx = OrganInferenceCtx {
        caps: capabilities,
        growth: finite_unit(growth_bias),
        mobility: finite_unit(mobility_bias),
        branching: finite_unit(branching_bias),
        robust: finite_unit(resilience),
        bio: finite_non_negative(biomass),
        progress: finite_unit(growth_progress),
        viability: finite_non_negative(viability),
    };
    let mut manifest = OrganManifest::new(stage);

    match stage {
        LifecycleStage::Dormant => {
            push_if_nonzero(&mut manifest, OrganRole::Core, 1, 0.3);
        }
        LifecycleStage::Emerging => {
            push_if_nonzero(&mut manifest, OrganRole::Stem, 1, 0.5);
            if has_capability(ctx.caps, crate::layers::CapabilitySet::GROW) {
                push_if_nonzero(&mut manifest, OrganRole::Bud, 1, 0.2);
            }
            if has_capability(ctx.caps, crate::layers::CapabilitySet::ROOT) {
                push_if_nonzero(&mut manifest, OrganRole::Root, 1, 0.3);
            }
        }
        LifecycleStage::Growing => {
            push_vegetative_roles(
                &mut manifest,
                &ctx,
                0.7 + ctx.progress * 0.3,
                infer_leaf_count(ctx.bio, ctx.growth),
                infer_thorn_count(ctx.bio, ctx.robust),
                false,
            );
        }
        LifecycleStage::Mature => {
            push_vegetative_roles(
                &mut manifest,
                &ctx,
                1.0,
                MAX_ORGAN_INSTANCE_COUNT,
                MAX_ORGAN_INSTANCE_COUNT,
                true,
            );
        }
        LifecycleStage::Reproductive => {
            // Base madura sin locomoción para priorizar roles reproductivos bajo presupuesto.
            push_vegetative_roles(
                &mut manifest,
                &OrganInferenceCtx {
                    caps: ctx.caps & !crate::layers::CapabilitySet::MOVE,
                    ..ctx
                },
                1.0,
                MAX_ORGAN_INSTANCE_COUNT,
                MAX_ORGAN_INSTANCE_COUNT,
                false,
            );
            push_reproductive_roles(&mut manifest, &ctx);
            if has_capability(ctx.caps, crate::layers::CapabilitySet::MOVE) {
                let limbs = infer_limb_count(ctx.bio, ctx.mobility);
                push_if_nonzero(&mut manifest, OrganRole::Limb, limbs, 0.6);
                push_if_nonzero(&mut manifest, OrganRole::Fin, limbs, 0.4);
            }
        }
        LifecycleStage::Declining => {
            let decline =
                ((ctx.viability / LIFECYCLE_DECLINING_VIABILITY) * DECLINING_ORGAN_FALLOFF).clamp(0.0, 1.0);
            push_if_nonzero(&mut manifest, OrganRole::Stem, 1, (0.7 + ctx.progress * 0.3) * decline);
            if has_capability(ctx.caps, crate::layers::CapabilitySet::PHOTOSYNTH) {
                let leaf_count =
                    count_to_u8_floor(infer_leaf_count(ctx.bio, ctx.growth) as f32 * decline).min(MAX_ORGAN_INSTANCE_COUNT);
                push_if_nonzero(&mut manifest, OrganRole::Leaf, leaf_count, 0.5 * ctx.growth * decline);
            }
            if has_capability(ctx.caps, crate::layers::CapabilitySet::ROOT) {
                let root_count =
                    count_to_u8_floor(infer_root_count(ctx.bio, ctx.growth) as f32 * decline).max(1);
                push_if_nonzero(&mut manifest, OrganRole::Root, root_count, 0.4 * decline);
            }
            if has_capability(ctx.caps, crate::layers::CapabilitySet::BRANCH) {
                let thorn_count =
                    count_to_u8_floor(infer_thorn_count(ctx.bio, ctx.robust) as f32 * ctx.robust * decline);
                push_if_nonzero(&mut manifest, OrganRole::Thorn, thorn_count, 0.2 * ctx.robust * decline);
            }
            if has_capability(ctx.caps, crate::layers::CapabilitySet::SENSE) {
                push_if_nonzero(&mut manifest, OrganRole::Sensory, 1, 0.15 * decline);
            }
            if has_capability(ctx.caps, crate::layers::CapabilitySet::ARMOR) {
                push_if_nonzero(&mut manifest, OrganRole::Shell, 1, 0.3 * ctx.robust * decline);
            }
        }
    }

    manifest
}

/// Deriva inputs normalizados para `infer_organ_manifest` desde estado agregado.
#[inline]
pub fn organ_manifest_inputs_from_state(qe_norm: f32, growth_efficiency: f32, biomass: f32) -> (f32, f32) {
    let growth_progress = finite_unit(finite_non_negative(biomass) / 3.0);
    let viability = finite_unit(finite_unit(qe_norm) * 0.7 + finite_unit(growth_efficiency) * 0.3);
    (growth_progress, viability)
}

const _: () = assert!(MAX_ORGAN_INSTANCE_COUNT as usize == crate::layers::MAX_ORGANS_PER_ENTITY);
