pub use components::*;

use amethyst::{
    ecs::{
        System,
        WriteStorage,
        ReadStorage,
        Read,
        Join,
        Entities,
    },
    core::{
        math::{Vector2},
        Time,
        Transform,
    },
};

pub const G: f32 = 0.001;    // Strength of gravity

pub mod components {
    use amethyst::{
        ecs::{Component, DenseVecStorage},
        core::math::{Vector2},
    };

    #[derive(Shrinkwrap, Component)]
    #[shrinkwrap(mutable)]
    pub struct Velocity(pub Vector2<f32>);
    
    #[derive(Shrinkwrap, Component)]
    #[shrinkwrap(mutable)]
    pub struct Mass(pub f32);
    
    #[derive(Shrinkwrap, Component)]
    #[shrinkwrap(mutable)]
    pub struct Force(pub Vector2<f32>); // Also acceleration. If the object has no mass, assume m = 1, so F = a
}

pub struct VelocitySystem;

impl<'a> System<'a> for VelocitySystem {
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (time, mut transforms, velocities): Self::SystemData) {
        let dt= time.fixed_seconds();

        for (transform, velocity) in (&mut transforms, &velocities).join() {
            transform.prepend_translation_x(velocity.x * dt);
            transform.prepend_translation_y(velocity.y * dt);
        }
    }
}

pub struct GravitySystem;

impl<'a> System<'a> for GravitySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Mass>,
        WriteStorage<'a, Force>,
    );

    fn run(&mut self, (entities, transforms, masses, mut forces): Self::SystemData) {
        for (entity, force, mass, transform) in (&entities, &mut forces, &masses, &transforms).join() {
            for (other_entity, other_mass, other_transform) in (&entities, &masses, &transforms).join() {
                if entity.id() != other_entity.id() { // If not the same planet
                    // F = GMm/r^2
                    // F_vec = (GMm/r^2) r_hat = (GMm/r^3) r_vec
                    // r is vector from this object to other object
                    let r_vec_3 = other_transform.translation() - transform.translation();
                    let r_vec = Vector2::new(r_vec_3.x, r_vec_3.y); // Convert to 2D
                    let distance_cubed = r_vec.norm().powi(3);
                    let grav_force = (G * mass.0 * other_mass.0/distance_cubed) * r_vec;

                    force.0 += grav_force;
                }
            }
        }
    }
}

pub struct ForceSystem;

impl<'a> System<'a> for ForceSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Mass>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Force>,
    );

    fn run(&mut self, (time, masses, mut velocities, mut forces): Self::SystemData) {
        let dt = time.fixed_seconds();
        // Update velocities with a force
        // F = ma, a = F/m, a = dv/dt, dv = a dt
        for (velocity, force, mass) in (&mut velocities, &forces, &masses).join() {
            velocity.0 += (force.0/mass.0) * dt;
        }
        // For objects with no mass, assume mass is 1
        for (velocity, force, _) in (&mut velocities, &forces, !&masses).join() {
            velocity.0 += force.0 * dt;
        }

        // Reset all forces to 0
        for force in (&mut forces).join() {
            force.x = 0.0;
            force.y = 0.0;
        }
    }
}

pub struct CollisionSystem;


