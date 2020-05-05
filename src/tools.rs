use std::f32::consts::PI;

pub fn volume_of_sphere(r: f32) -> f32 {
    4.0/3.0 * PI * r.powi(3)
}

pub fn inverse_volume_of_sphere(v: f32) -> f32 {
    (3.0/(4.0 * PI) * v).powf(1.0/3.0)
}

#[inline]
pub fn circular_orbit_speed(parent_mass: f32, radius: f32) -> f32 {
    (crate::systems::physics::G * parent_mass/radius).sqrt()
}