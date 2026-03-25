use crate::blueprint::{AbilityDef, AlchemicalAlmanac, FormulaValidator};
use crate::layers::will::Grimoire;

/// Compila y valida una ability antes de habilitarla en el grimorio.
pub fn compile_and_enable_ability(
    grimoire: &mut Grimoire,
    mut def: AbilityDef,
    validator: &FormulaValidator,
    almanac: &AlchemicalAlmanac,
) -> Result<u32, Vec<String>> {
    validator.validate(&mut def, almanac)?;
    if !grimoire.push_ability(def.to_slot()) {
        return Err(vec!["Grimoire at max abilities".to_string()]);
    }
    Ok(def.checksum)
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
            banned_freq_ranges: vec![],
        }
    }

    fn mk_almanac() -> AlchemicalAlmanac {
        AlchemicalAlmanac::default()
    }

    fn mk_ability() -> AbilityDef {
        AbilityDef {
            name: "Sodium Burst".to_string(),
            description: "Reactive projectile".to_string(),
            cost_qe: 100.0,
            cooldown_estimate: 0.2,
            output: AbilityOutput::Projectile {
                element_id: ElementId::from_name("Ignis"),
                radius: 1.0,
                speed: 20.0,
                effect: None,
            },
            frequency_override: None,
            phase_offset: None,
            max_range: 10.0,
            max_duration_s: 2.0,
            validated: false,
            checksum: 0,
        }
    }

    #[test]
    fn compile_and_enable_ability_inserts_only_validated() {
        let mut grimoire = Grimoire::default();
        let validator = mk_validator();
        let almanac = mk_almanac();

        let result = compile_and_enable_ability(&mut grimoire, mk_ability(), &validator, &almanac);
        assert!(result.is_ok());
        assert_eq!(grimoire.abilities().len(), 1);
    }

    #[test]
    fn compile_and_enable_ability_rejected_does_not_insert() {
        let mut grimoire = Grimoire::default();
        let validator = mk_validator();
        let almanac = mk_almanac();
        let mut invalid = mk_ability();
        invalid.cost_qe = 9999.0;

        let result = compile_and_enable_ability(&mut grimoire, invalid, &validator, &almanac);
        assert!(result.is_err());
        assert!(grimoire.abilities().is_empty());
    }
}
