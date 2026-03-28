//! Shared CLI argument parsing + display helpers for binaries.
//!
//! Eliminates duplication of `parse_arg`, `find_arg`, and archetype labeling
//! across 10+ binaries.

/// Parse a numeric CLI flag. Returns `default` if absent or unparseable.
pub fn parse_arg(args: &[String], flag: &str, default: i64) -> i64 {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(default)
}

/// Find a string CLI flag value. Returns `None` if absent.
pub fn find_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

/// Human-readable archetype label from u8 code.
pub fn archetype_label(archetype: u8) -> &'static str {
    match archetype {
        1 => "flora",
        2 => "fauna",
        3 => "cell",
        4 => "virus",
        _ => "inert",
    }
}

/// Human-readable trophic class label from u8 code.
pub fn trophic_label(trophic: u8) -> &'static str {
    match trophic {
        0 => "prod",
        1 => "herb",
        2 => "omni",
        3 => "carn",
        4 => "detr",
        _ => "?",
    }
}

/// Resolve preset name to UniversePreset.
pub fn resolve_preset(name: &str) -> crate::use_cases::presets::UniversePreset {
    match name.to_lowercase().as_str() {
        "jupiter" => crate::use_cases::presets::JUPITER,
        "mars"    => crate::use_cases::presets::MARS,
        "eden"    => crate::use_cases::presets::EDEN,
        "hell"    => crate::use_cases::presets::HELL,
        "random"  => crate::use_cases::presets::UniversePreset::from_seed(42),
        _         => crate::use_cases::presets::EARTH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_arg_returns_default_when_missing() {
        let args: Vec<String> = vec!["bin".into()];
        assert_eq!(parse_arg(&args, "--gens", 100), 100);
    }

    #[test]
    fn parse_arg_extracts_value() {
        let args: Vec<String> = vec!["bin".into(), "--gens".into(), "500".into()];
        assert_eq!(parse_arg(&args, "--gens", 100), 500);
    }

    #[test]
    fn find_arg_returns_none_when_missing() {
        let args: Vec<String> = vec!["bin".into()];
        assert_eq!(find_arg(&args, "--preset"), None);
    }

    #[test]
    fn find_arg_extracts_string() {
        let args: Vec<String> = vec!["bin".into(), "--preset".into(), "jupiter".into()];
        assert_eq!(find_arg(&args, "--preset"), Some("jupiter".to_string()));
    }

    #[test]
    fn archetype_label_covers_all() {
        assert_eq!(archetype_label(0), "inert");
        assert_eq!(archetype_label(1), "flora");
        assert_eq!(archetype_label(2), "fauna");
        assert_eq!(archetype_label(3), "cell");
        assert_eq!(archetype_label(4), "virus");
        assert_eq!(archetype_label(255), "inert");
    }
}
