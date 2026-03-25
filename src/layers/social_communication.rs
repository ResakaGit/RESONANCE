use bevy::prelude::*;

/// Rol dentro de un pack social.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackRole {
    Leader,
    Member,
    Juvenile,
}

/// Membresía a un pack social (D6). Max 3 fields.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct PackMembership {
    pub pack_id: u32,
    pub role: PackRole,
    pub joined_tick: u32,
}

impl PackMembership {
    pub fn new(pack_id: u32, role: PackRole, joined_tick: u32) -> Self {
        Self { pack_id, role, joined_tick }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_membership_new_preserves_fields() {
        let m = PackMembership::new(42, PackRole::Leader, 100);
        assert_eq!(m.pack_id, 42);
        assert_eq!(m.role, PackRole::Leader);
        assert_eq!(m.joined_tick, 100);
    }

    #[test]
    fn pack_role_equality() {
        assert_eq!(PackRole::Member, PackRole::Member);
        assert_ne!(PackRole::Leader, PackRole::Juvenile);
    }
}
