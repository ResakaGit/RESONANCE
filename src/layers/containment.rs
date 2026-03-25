use bevy::prelude::*;

/// Contexto de contención inferido por superposición espacial (no settable por gameplay).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct ContainedIn {
    /// La entidad host que me contiene (por Capa 6: `AmbientPressure` + Capa 1: `SpatialVolume`).
    pub host: Entity,

    /// Cómo estoy contenido: conducción (Surface), convección (Immersed) o radiación (Radiated).
    pub contact: ContactType,
}

/// Contrato escalar para el canal dominante de transferencia.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum ContactType {
    /// Intersección / proximidad de frontera. Transferencia dominante por conducción.
    Surface,
    /// Dentro del volumen pero con la “frontera” lo suficientemente lejos.
    /// Transferencia dominante por convección.
    Immersed,
    /// Fuera del volumen pero dentro del rango de influencia.
    /// Transferencia dominante por radiación.
    Radiated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    #[test]
    fn contained_in_roundtrip() {
        let host = Entity::from_raw(7);
        let c = ContainedIn {
            host,
            contact: ContactType::Immersed,
        };
        assert_eq!(c.host, host);
        assert_eq!(c.contact, ContactType::Immersed);
    }

    #[test]
    fn contact_type_variants_distinct() {
        assert_ne!(ContactType::Surface, ContactType::Immersed);
        assert_ne!(ContactType::Immersed, ContactType::Radiated);
    }

    #[test]
    fn contained_in_copy_preserves_bits() {
        let c = ContainedIn {
            host: Entity::from_raw(1),
            contact: ContactType::Surface,
        };
        let c2 = c;
        assert_eq!(c, c2);
    }
}
