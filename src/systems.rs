use amethyst::{
    ecs::{System, SystemData, Write, WriteStorage, ReaderId, Read, Entities, World},
    core::{
        transform::Transform, 
        SystemDesc,
        math::{Vector2},
    },
    renderer::SpriteRender,
    shrev::EventChannel,
    shred::ResourceId,
    input::{InputHandler, StringBindings},
};
use crate::components::*;
use crate::resources::*;
use crate::events::BodyCreationEvent;


// System for creating bodies from an events channel.
pub struct BodyCreationSystem {
    reader_id: ReaderId<BodyCreationEvent>,
}

impl<'a> System<'a> for BodyCreationSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventChannel<BodyCreationEvent>>,
        BodyCreationData<'a>,
    );

    fn run(
        &mut self,
        (
            mut entities,
            events,
            mut body_creation_sys_data,
        ): Self::SystemData
    ) {
        for creation_event in events.read(&mut self.reader_id) {
            info!("Creating body: {:?}", creation_event);
            creation_event.build_entity(
                &mut entities,
                &mut body_creation_sys_data,
            );
        }
    }
}

impl BodyCreationSystem {
    pub fn new(reader_id: ReaderId<BodyCreationEvent>) -> Self {
        Self { reader_id }
    }
}

pub struct BodyCreationSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, BodyCreationSystem> for BodyCreationSystemDesc {
    fn build(self, world: &mut World) -> BodyCreationSystem {
        <BodyCreationSystem as System<'_>>::SystemData::setup(world);
    
        let reader_id = world.fetch_mut::<EventChannel<BodyCreationEvent>>().register_reader();
        BodyCreationSystem::new(reader_id)
    }
}

#[derive(SystemData)]
pub struct BodyCreationData<'a> { // Data needed to create new body
    pub body_type: WriteStorage<'a, BodyType>,
    pub transforms: WriteStorage<'a, Transform>,
    pub velocities: WriteStorage<'a, Velocity>,
    pub colliders: WriteStorage<'a, Collider>,
    pub forces: WriteStorage<'a, Force>,
    pub masses: WriteStorage<'a, Mass>,
    pub renders_resource: Read<'a, SpriteRenders>,
    pub render_storage: WriteStorage<'a, SpriteRender>,
}


pub struct InputParsingSystem;

impl<'a> System<'a> for InputParsingSystem {
    type SystemData = (
        Read<'a, InputHandler<StringBindings>>,
        Write<'a, MouseInfo>,
        Write<'a, EventChannel<BodyCreationEvent>>,
    );

    fn run(&mut self, (input, mut mouse_info, mut body_creation_channel): Self::SystemData) {
        use crate::CAMERA_DIMS;

        if input.action_is_down("add_planet").unwrap_or(false) && !mouse_info.is_down {
            mouse_info.is_down = true;
            let pos = input.mouse_position().unwrap();
            mouse_info.click_pos = Some(Vector2::new(pos.0, CAMERA_DIMS.1 - pos.1));
        }

        // if no longer down
        if !input.action_is_down("add_planet").unwrap_or(false) && mouse_info.is_down && mouse_info.click_pos.is_some() {
            if let Some(curr_pos) = input.mouse_position() {
                let curr_pos = Vector2::new(curr_pos.0, CAMERA_DIMS.1 - curr_pos.1);

                let original_click_pos = mouse_info.click_pos.unwrap();
                let mouse_spawn_radius = input.axis_value("planet_size").unwrap().max(1.0);
    
                mouse_info.is_down = false;
    
                let d_pos = original_click_pos - curr_pos;
                let mass = crate::tools::volume_of_sphere(mouse_spawn_radius) * crate::entities::body::PLANET_DENSITY;
    
                body_creation_channel.single_write(BodyCreationEvent {
                    body_type: BodyType::from_mass(mass),
                    position: original_click_pos.into(),
                    velocity: d_pos,
                    mass,
                    radius: mouse_spawn_radius,
                });
            }
        }
    }
}



pub mod physics {
    use amethyst::{
        ecs::{
            System, SystemData,
            WriteStorage, ReadStorage, Read, Write, ReaderId,
            Join,
            Entities, Entity,
            World,
        },
        core::{
            SystemDesc,
            math::{Vector2, Point2},
            Time,
            Transform,
        },
        shrev::EventChannel,
    };
    use std::collections::HashSet;
    
    use crate::components::*;
    use crate::events::*;
    
    pub const G: f32 = 0.0001;    // Strength of gravity
    
    pub struct VelocitySystem;
    
    impl<'a> System<'a> for VelocitySystem {
        type SystemData = (
            Read<'a, Time>,
            WriteStorage<'a, Transform>,
            ReadStorage<'a, Velocity>,
        );
    
        fn run(&mut self, (time, mut transforms, velocities): Self::SystemData) {
            let dt= time.delta_seconds();
    
            for (transform, velocity) in (&mut transforms, &velocities).join() {
                transform.prepend_translation_x(velocity.x * dt);
                transform.prepend_translation_y(velocity.y * dt);
            }
        }
    }
    
    pub struct GravitySystem;
    
    impl<'a> System<'a> for GravitySystem {
        type SystemData = (
            ReadStorage<'a, Transform>,
            ReadStorage<'a, Mass>,
            WriteStorage<'a, Force>,
        );
    
        fn run(&mut self, (transforms, masses, mut forces): Self::SystemData) {
            struct GravBody<'a> {   // Helper struct to make things less confusing
                force: &'a mut Force,
                mass: &'a Mass,
                transform: &'a Transform,
            }

            let mut grav_bodies: Vec<GravBody> = 
                (&mut forces, &masses, &transforms).join()
                    .map(|(force, mass, transform)| GravBody {
                        force,
                        mass,
                        transform,
                    }).collect();

            // Bodies that experience gravity are collected so that an optimised approach can be used.
            // Since the force experienced between to planets is equal and _opposite_ for the other planet,
            // we only need to calculate the force between a pair.

            let len = grav_bodies.len();
            for i in 0..len-1 { // For every body except from last
                for j in i+1..len {   // For every body not done (i) onwards
                    // F = GMm/r^2
                    // F_vec = (GMm/r^2) r_hat = (GMm/r^3) r_vec
                    // r is vector from this object to other object
                    let r_vec_3 = grav_bodies[j].transform.translation() - grav_bodies[i].transform.translation();
                    let r_vec = Vector2::new(r_vec_3.x, r_vec_3.y); // Convert to 2D
                    let distance_cubed = r_vec.norm().powi(3);
                    // grav_force will be experienced by both
                    let grav_force = (G * grav_bodies[i].mass.0 * grav_bodies[j].mass.0/distance_cubed) * r_vec;

                    grav_bodies[i].force.0 += grav_force;
                    grav_bodies[j].force.0 -= grav_force;   // -= cause force is applied in opposite direction
                }
            }

            // for (entity, force, mass, transform) in  {
            //     for (_other_entity, other_mass, other_transform) in
            //         (&entities, &masses, &transforms).join()
            //             .filter(|(e, _, _)| e.id() != entity.id()) // If not the same planet
            //     {
            //         // F = GMm/r^2
            //         // F_vec = (GMm/r^2) r_hat = (GMm/r^3) r_vec
            //         // r is vector from this object to other object
            //         let r_vec_3 = other_transform.translation() - transform.translation();
            //         let r_vec = Vector2::new(r_vec_3.x, r_vec_3.y); // Convert to 2D
            //         let distance_cubed = r_vec.norm().powi(3);
            //         let grav_force = (G * mass.0 * other_mass.0/distance_cubed) * r_vec;
    
            //         force.0 += grav_force;
            //     }
            // }
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
            let dt = time.delta_seconds();
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
    
    pub struct CollisionDetectionSystem;
    
    impl<'a> System<'a> for CollisionDetectionSystem {
        type SystemData = (
            Entities<'a>,
            Write<'a, EventChannel<CollisionEvent>>,
            ReadStorage<'a, Transform>,        // Need write access to all these to add new planets
            ReadStorage<'a, Collider>,
        );
    
        fn run(
            &mut self, 
            (
                entities,
                mut collision_event_channel,
                transforms,
                colliders,
            ): Self::SystemData,
        ) {
            let mut collision_groups: Vec<HashSet<Entity>> = Vec::new();
    
            for (entity, transform, collider) in (&entities, &transforms, &colliders).join() {
                for (other_entity, other_transform, other_collider) in 
                    (&entities, &transforms, &colliders).join()
                        .filter(|(e, _, _)| e.id() != entity.id())    // If the entity is the same as the first entity, don't collide with itself
                {
                    if collider.is_colliding_with(transform, other_collider, other_transform) {
                        Self::put_in_collision_group(&mut collision_groups, entity, other_entity);
                    }
                }
            }

            // Send event to collision channel
            collision_event_channel.iter_write(
                collision_groups
                    .into_iter()
                    .map(|group| {
                        CollisionEvent {
                            group,
                        }
                    })
            );
        }
    }
    
    impl CollisionDetectionSystem {
        fn put_in_collision_group(collision_groups: &mut Vec<HashSet<Entity>>, e1: Entity, e2: Entity) {
            // If either entity is in an existing group, then add the other to that group.
            // Otherwise, make a new group.
            let mut existing_group_found = false;
            let mut already_paired = false; // If they were paired
    
            // Check for existing group
            for group in collision_groups.iter_mut() {
                let (contains1, contains2) = (group.contains(&e1), group.contains(&e2));
                if contains1 || contains2 {  // If it contains either one, but not both
                    if contains1 && contains2 { // Already BOTH in group, so ignore
                        info!("{:?} and {:?} already paired.", e1, e2);
                        already_paired = true;
                    } else {
                        if contains1 {
                            info!("Inserting {:?} into group {:?}", e2, group);
                            if !group.insert(e2) { error!("Entity {} was already in collision group {:?}.", e2.id(), group) }
                        } else if contains2 {
                            info!("Inserting {:?} into group {:?}", e1, group);
                            if !group.insert(e1) { error!("Entity {} was already in collision group {:?}.", e1.id(), group) }
                        }
                    }
                    existing_group_found = true;
                    break;
                }
            }
    
            // Make a new group if there was no existing group
            if !existing_group_found && !already_paired {
                let mut set = HashSet::with_capacity(2);
                set.insert(e1);
                set.insert(e2);
                collision_groups.push(set);
            }
        }
    }

    pub struct CollisionProcessingSystem {
        reader_id: ReaderId<CollisionEvent>,
    }

    impl<'a> System<'a> for CollisionProcessingSystem {
        type SystemData = (
            Entities<'a>,
            Read<'a, EventChannel<CollisionEvent>>,
            Write<'a, EventChannel<BodyCreationEvent>>,
            ReadStorage<'a, Transform>,
            ReadStorage<'a, Velocity>,
            ReadStorage<'a, Mass>,
        );

        fn run(
            &mut self,
            (
                entities,
                collision_event_channel,
                mut body_creation_event_channel,
                transforms,
                velocities,
                masses,
            ): Self::SystemData
        ) {
            for event in collision_event_channel.read(&mut self.reader_id) {
                let group: &HashSet<Entity> = &event.group;

                info!("CollisionEvent: {:?}", event);
        
                // Find centre of mass = new position
                // r_com = SUM( m * r ) where r is position vector
                let mut r_m_sum: Vector2<f32> = Vector2::zeros();
                let mut mass_sum: f32 = 0.0;
    
                // Momentum before = momentum after
                let mut momentum_sum: Vector2<f32> = Vector2::zeros();

                for entity in group {
                    // Get mass. Assume mass is 1 if no mass.
                    let mass = masses.get(*entity)
                        .unwrap_or(&Mass(1.0));
                    let velocity: Velocity = velocities.get(*entity).copied()
                        .unwrap_or(Velocity(Vector2::zeros()));
    
                    let translation = transforms.get(*entity).unwrap().translation();
                    let r = Vector2::new(translation.x, translation.y);
    
                    r_m_sum += r * mass.0;
                    mass_sum += mass.0;
                    momentum_sum += velocity.0 * mass.0;
    
                    entities.delete(*entity).expect("Could not delete entity.");
                }
    
                // p = mv, v = p/m
                let r_com: Point2<f32> = Point2::from(r_m_sum/mass_sum as f32);
                let vel = momentum_sum/mass_sum;

                body_creation_event_channel.single_write(BodyCreationEvent {
                    body_type: BodyType::from_mass(mass_sum),
                    position: r_com,
                    velocity: vel,
                    mass: mass_sum,
                    radius: crate::tools::inverse_volume_of_sphere(mass_sum/crate::entities::body::PLANET_DENSITY),
                });
            }
        }
    }

    impl<'a> CollisionProcessingSystem {
        pub fn new(reader_id: ReaderId<CollisionEvent>) -> Self {
            Self { reader_id: reader_id }
        }
    }

    pub struct CollisionProcessingSystemDesc;

    impl<'a, 'b> SystemDesc<'a, 'b, CollisionProcessingSystem> for CollisionProcessingSystemDesc {
        fn build(self, world: &mut World) -> CollisionProcessingSystem {
            <CollisionProcessingSystem as System<'_>>::SystemData::setup(world);
        
            let reader_id = world.fetch_mut::<EventChannel<CollisionEvent>>().register_reader();
            CollisionProcessingSystem::new(reader_id)
        }
    }
}
