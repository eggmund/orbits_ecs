
pub mod planet {
    use amethyst::{
        prelude::*,
        core::transform::Transform,
        core::math::{Vector2, Point2, Vector3},
        ecs::{
            World,
        },
        renderer::{
            SpriteRender, SpriteSheet,
        },
        assets::Handle,
    };
    use rand::prelude::*;
    use rand::rngs::ThreadRng;
    
    use std::f32::consts::PI;

    const PLANET_SPRITE_RADIUS: f32 = 32.0/2.0;    // Radius of default sprite = width/2.0
    const PLANET_SPRITE_RATIO: f32 = 1.0/PLANET_SPRITE_RADIUS;

    fn calculate_planet_mass(radius: f32) -> f32 {
        const PLANET_DENSITY: f32 = 5000.0;
        4.0/3.0 * PI * radius.powi(3) * PLANET_DENSITY
    }

    // Returns the magnitude of the velocity (speed) needed for a circular orbit around another planet
    // Orbit is circular when the kinetic energy does not change.
    // K = GMm/2r  -- Derived from centripetal force (in circular motion) = gravitational force
    // GMm/2r = 1/2 mv^2
    // GM/2r = 1/2 v^2
    // sqrt(GM/r) = v
    #[inline]
    fn circular_orbit_speed(parent_mass: f32, radius: f32) -> f32 {
        (crate::physics::G * parent_mass/radius).sqrt()
    }
    
    pub fn add_planet(
        world: &mut World,
        sprite_sheet_handle: Handle<SpriteSheet>,
        pos: Point2<f32>,
        vel: Vector2<f32>,
        radius: f32
    ) {
        // Get the planet sprite from the spritesheet
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 0,
        };
    
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(PLANET_SPRITE_RATIO * radius, PLANET_SPRITE_RATIO * radius, 0.0));
        transform.set_translation_xyz(pos.x, pos.y, 0.0);
    
        // m = v * density
        let mass = self::calculate_planet_mass(radius);
    
        world.create_entity()
            .with(sprite_render)
            .with(transform)
            .with(crate::physics::Velocity(vel))
            .with(crate::physics::Mass(mass))
            .with(crate::physics::Force(Vector2::zeros()))
            .build();
    }

    pub fn add_planet_with_rings(
        world: &mut World,
        spritesheet_handle: Handle<SpriteSheet>,
        rand_thread: &mut ThreadRng,
        position: Point2<f32>,
        velocity: Vector2<f32>,
    
        main_planet_radius: f32,
        moon_num: usize,
        moon_orbit_radius_range: (f32, f32),    // Starting from surface of planet
        moon_body_radius_range: (f32, f32),
        orbit_direction_clockwise: bool,  // anticlockwise = false, clockwise = true
    ) {
        self::add_planet(world, spritesheet_handle.clone(), position.clone(), velocity.clone(), main_planet_radius);  // Add main planet
    
        let main_planet_mass = self::calculate_planet_mass(main_planet_radius);
        let frame_velocity = velocity;
    
        for _ in 0..moon_num {
            let orbit_radius = main_planet_radius + rand_thread.gen_range(moon_orbit_radius_range.0, moon_orbit_radius_range.1);
            let orbit_speed = self::circular_orbit_speed(main_planet_mass, orbit_radius);
            let start_angle = rand_thread.gen_range(0.0, PI * 2.0);      // Angle from main planet to moon
            let start_pos = Point2::new(orbit_radius * start_angle.cos(), orbit_radius * start_angle.sin());   // Position on circle orbit where planet will start

            let vel_angle = if orbit_direction_clockwise {
                start_angle + PI/2.0
            } else {
                start_angle - PI/2.0
            };
            let start_velocity = Vector2::new(orbit_speed * vel_angle.cos(), orbit_speed * vel_angle.sin());
            let moon_radius = rand_thread.gen_range(moon_body_radius_range.0, moon_body_radius_range.1);
    
            self::add_planet(
                world,
                spritesheet_handle.clone(),
                Point2::new(position.x + start_pos.x, position.y + start_pos.y),
                start_velocity + frame_velocity,  // Add velocity of main planet
                moon_radius,
            );
        }
    }
}
