
pub mod body {
    use std::f32::consts::PI;
    const PLANET_SPRITE_RADIUS: f32 = 32.0/2.0;    // Radius of default sprite = width/2.0
    pub const PLANET_SPRITE_RATIO: f32 = 1.0/PLANET_SPRITE_RADIUS;

    const STAR_MIN_RADIUS: f32 = 50.0;
    // Point at which planet becomes a star.
    const PLANET_STAR_MASS_BOUNDARY: f32 = 4.0/3.0 * PI * STAR_MIN_RADIUS * STAR_MIN_RADIUS * STAR_MIN_RADIUS * PLANET_DENSITY;
    pub const PLANET_DENSITY: f32 = 5000.0;

    // Returns the magnitude of the velocity (speed) needed for a circular orbit around another planet
    // Orbit is circular when the kinetic energy does not change.
    // K = GMm/2r  -- Derived from centripetal force (in circular motion) = gravitational force
    // GMm/2r = 1/2 mv^2
    // GM/2r = 1/2 v^2
    // sqrt(GM/r) = v


    // pub fn add_body_with_satellites(
    //     world: &mut World,
    //     sprite_renders: &SpriteRender,
    //     rand_thread: &mut ThreadRng,
    //     position: Point2<f32>,
    //     velocity: Vector2<f32>,
    
    //     main_body_radius: f32,
    //     satellite_num: usize,
    //     satellite_orbit_radius_range: (f32, f32),    // Starting from surface of body
    //     satellite_body_radius_range: (f32, f32),
    //     orbit_direction_clockwise: bool,  // anticlockwise = false, clockwise = true
    // ) {
    //     self::add_body(world.create_entity(), sprite_render, position.clone(), velocity.clone(), main_body_radius);  // Add main planet
    
    //     let main_body_mass = crate::tools::volume_of_sphere(main_body_radius) * PLANET_DENSITY;
    //     let frame_velocity = velocity;
    
    //     for _ in 0..satellite_num {
    //         let orbit_radius = main_body_radius + rand_thread.gen_range(satellite_orbit_radius_range.0, satellite_orbit_radius_range.1);
    //         let orbit_speed = self::circular_orbit_speed(main_body_mass, orbit_radius);
    //         let start_angle = rand_thread.gen_range(0.0, PI * 2.0);      // Angle from main body to satellite
    //         let start_pos = Point2::new(orbit_radius * start_angle.cos(), orbit_radius * start_angle.sin());   // Position on circle orbit where satellite will start

    //         let vel_angle = if orbit_direction_clockwise {
    //             start_angle + PI/2.0
    //         } else {
    //             start_angle - PI/2.0
    //         };
    //         let start_velocity = Vector2::new(orbit_speed * vel_angle.cos(), orbit_speed * vel_angle.sin());
    //         let satellite_radius = rand_thread.gen_range(satellite_body_radius_range.0, satellite_body_radius_range.1);
    
    //         self::add_body(
    //             world.create_entity(),
    //             sprite_render,
    //             Point2::new(position.x + start_pos.x, position.y + start_pos.y),
    //             start_velocity + frame_velocity,  // Add velocity of main body
    //             satellite_radius,
    //         );
    //     }
    // }
}
