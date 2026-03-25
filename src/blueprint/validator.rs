use crate::blueprint::{AbilityDef, AlchemicalAlmanac};
use crate::layers::AbilityOutput;

#[derive(Clone, Debug)]
pub struct FormulaValidator {
    pub max_cost_budget: f32,
    pub max_damage_per_hit: f32,
    pub max_aoe_radius: f32,
    pub min_cooldown: f32,
    pub max_range: f32,
    pub max_duration_s: f32,
    pub banned_freq_ranges: Vec<(f32, f32)>,
}

impl FormulaValidator {
    /// Valida una ability y, si pasa, la marca usable con checksum determinista.
    pub fn validate(
        &self,
        def: &mut AbilityDef,
        almanac: &AlchemicalAlmanac,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if def.name.trim().is_empty() {
            errors.push("name must not be empty".to_string());
        }
        if def.description.trim().is_empty() {
            errors.push("description must not be empty".to_string());
        }
        if def.cost_qe <= 0.0 {
            errors.push("cost_qe must be > 0".to_string());
        }
        if def.cost_qe > self.max_cost_budget {
            errors.push(format!(
                "cost_qe {} exceeds max_cost_budget {}",
                def.cost_qe, self.max_cost_budget
            ));
        }
        if def.max_range <= 0.0 {
            errors.push("max_range must be > 0".to_string());
        }
        if def.max_range > self.max_range {
            errors.push(format!(
                "max_range {} exceeds allowed {}",
                def.max_range, self.max_range
            ));
        }
        if def.max_duration_s <= 0.0 {
            errors.push("max_duration_s must be > 0".to_string());
        }
        if def.max_duration_s > self.max_duration_s {
            errors.push(format!(
                "max_duration_s {} exceeds allowed {}",
                def.max_duration_s, self.max_duration_s
            ));
        }
        if def.cooldown_estimate < self.min_cooldown {
            def.cooldown_estimate = self.min_cooldown;
        }

        if let Some(freq) = def.frequency_override {
            if self
                .banned_freq_ranges
                .iter()
                .any(|(lo, hi)| freq >= *lo && freq <= *hi)
            {
                errors.push(format!("frequency_override {} is banned", freq));
            }
            if almanac.find_stable_band(freq).is_none() {
                errors.push(format!(
                    "frequency_override {} is outside stable bands",
                    freq
                ));
            }
        }

        match &def.output {
            AbilityOutput::Projectile { radius, .. } => {
                if *radius > self.max_aoe_radius {
                    errors.push(format!(
                        "projectile radius {} exceeds max_aoe_radius {}",
                        radius, self.max_aoe_radius
                    ));
                }
                if def.cost_qe > self.max_damage_per_hit {
                    errors.push(format!(
                        "projectile cost {} exceeds max_damage_per_hit {}",
                        def.cost_qe, self.max_damage_per_hit
                    ));
                }
            }
            AbilityOutput::Barrage { radius, count, .. } => {
                if *radius > self.max_aoe_radius {
                    errors.push(format!(
                        "barrage radius {} exceeds max_aoe_radius {}",
                        radius, self.max_aoe_radius
                    ));
                }
                let effective_hit_budget = def.cost_qe * (*count as f32);
                if effective_hit_budget > self.max_damage_per_hit {
                    errors.push(format!(
                        "barrage effective damage {} exceeds max_damage_per_hit {}",
                        effective_hit_budget, self.max_damage_per_hit
                    ));
                }
            }
            AbilityOutput::Summon { .. }
            | AbilityOutput::Transmute { .. }
            | AbilityOutput::SelfBuff { .. }
            | AbilityOutput::Zone { .. }
            | AbilityOutput::Pickup { .. } => {}
        }

        if errors.is_empty() {
            def.validated = true;
            def.checksum = checksum_ability(def);
            Ok(())
        } else {
            def.validated = false;
            def.checksum = 0;
            Err(errors)
        }
    }
}

pub fn checksum_ability(def: &AbilityDef) -> u32 {
    let serialized = serde_json::to_vec(def).unwrap_or_default();
    fxhash::hash32(&serialized)
}

#[cfg(test)]
mod tests {
    use crate::blueprint::ElementId;
    use crate::layers::AbilityOutput;

    use super::*;

    fn mk_validator() -> FormulaValidator {
        FormulaValidator {
            max_cost_budget: 200.0,
            max_damage_per_hit: 240.0,
            max_aoe_radius: 3.0,
            min_cooldown: 0.5,
            max_range: 20.0,
            max_duration_s: 15.0,
            banned_freq_ranges: vec![(666.0, 700.0)],
        }
    }

    fn mk_almanac() -> AlchemicalAlmanac {
        AlchemicalAlmanac::default()
    }

    fn mk_ability() -> AbilityDef {
        AbilityDef {
            name: "Sodium Burst".to_string(),
            description: "Sodium projectile with controlled instability".to_string(),
            cost_qe: 150.0,
            cooldown_estimate: 0.1,
            output: AbilityOutput::Projectile {
                element_id: ElementId::from_name("Ignis"),
                radius: 0.8,
                speed: 25.0,
                effect: None,
            },
            frequency_override: None,
            phase_offset: Some(0.0),
            max_range: 18.0,
            max_duration_s: 3.0,
            validated: false,
            checksum: 0,
        }
    }

    #[test]
    fn validate_rejects_too_expensive() {
        let validator = mk_validator();
        let almanac = mk_almanac();
        let mut ability = mk_ability();
        ability.cost_qe = 999.0;

        let result = validator.validate(&mut ability, &almanac);
        assert!(result.is_err());
        assert!(!ability.validated);
    }

    #[test]
    fn validate_rejects_incompatible_frequency() {
        let validator = mk_validator();
        let almanac = mk_almanac();
        let mut ability = mk_ability();
        ability.frequency_override = Some(100.0);

        let result = validator.validate(&mut ability, &almanac);
        assert!(result.is_err());
        assert!(!ability.validated);
    }

    #[test]
    fn validate_rejects_executions_limits() {
        let validator = mk_validator();
        let almanac = mk_almanac();
        let mut ability = mk_ability();
        ability.max_duration_s = 120.0;

        let result = validator.validate(&mut ability, &almanac);
        assert!(result.is_err());
        assert!(!ability.validated);
    }

    #[test]
    fn validate_accepts_valid_ability() {
        let validator = mk_validator();
        let almanac = mk_almanac();
        let mut ability = mk_ability();

        let result = validator.validate(&mut ability, &almanac);
        assert!(result.is_ok());
        assert!(ability.validated);
        assert!(ability.checksum != 0);
        assert!(ability.cooldown_estimate >= validator.min_cooldown);
    }

    #[test]
    fn validate_accepts_parameter_variants_with_distinct_checksum() {
        let validator = mk_validator();
        let almanac = mk_almanac();

        let mut a = mk_ability();
        let mut b = mk_ability();
        b.output = AbilityOutput::Projectile {
            element_id: ElementId::from_name("Ignis"),
            radius: 1.3,
            speed: 22.0,
            effect: None,
        };

        assert!(validator.validate(&mut a, &almanac).is_ok());
        assert!(validator.validate(&mut b, &almanac).is_ok());
        assert_ne!(a.checksum, b.checksum);
    }
}
