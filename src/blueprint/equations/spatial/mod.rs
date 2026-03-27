use crate::math_types::Vec2;

/// Distancia entre dos puntos 2D.
pub fn distance(pos_a: Vec2, pos_b: Vec2) -> f32 {
    (pos_a - pos_b).length()
}

/// ¿Dos esferas están colisionando? (distancia < suma de radios)
pub fn has_collision(pos_a: Vec2, radius_a: f32, pos_b: Vec2, radius_b: f32) -> bool {
    distance(pos_a, pos_b) < radius_a + radius_b
}
