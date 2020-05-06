use amethyst::{
    ecs::{System, Write}
};

pub mod physics {
    use amethyst::{
        ecs::{
            System, SystemData,
            WriteStorage, ReadStorage, Read, Write, ReaderId,
            Join,
            Entities, Entity,
            World,
            world::Builder,
        },
        core::{
            SystemDesc,
            math::{Vector2, Vector3},
            Time,
            Transform,
            EventReader,
            RunNowDesc,
        },
        DataInit,
        derive::SystemDesc,
        shrev::EventChannel,
        renderer::{SpriteRender, SpriteSheet},
        assets::{AssetStorage, Handle},
    };
    use std::collections::HashSet;
    
    use crate::components::{*, physics::*};
    use crate::resources;
    use crate::events::{CollisionEvent};
    
    pub const G: f32 = 0.0001;    // Strength of gravity
    
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
                for (other_entity, other_mass, other_transform) in
                    (&entities, &masses, &transforms).join()
                        .filter(|(e, _, _)| e.id() != entity.id()) // If not the same planet
                {
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
            Read<'a, resources::SpriteRenders>,
            WriteStorage<'a, SpriteRender>,
            // Need to add planet so needs write access to all these
            WriteStorage<'a, Transform>,
            // WriteStorage<'a, SpriteRender>,
            WriteStorage<'a, Velocity>,
            WriteStorage<'a, Collider>,
            WriteStorage<'a, Force>,
            WriteStorage<'a, Mass>,
        );

        fn run(
            &mut self,
            (
                entities,
                event_channel,
                sprite_renders,
                mut sprite_renders_storage,
                mut transforms,
                // mut sprite_render_storage,
                mut velocities,
                mut colliders,
                mut forces,
                mut masses,
            ): Self::SystemData
        ) {
            use ncollide2d::shape::Ball;

            let render = sprite_renders.planet.as_ref().unwrap().clone();

            for event in event_channel.read(&mut self.reader_id) {
                let group: &HashSet<Entity> = &event.group;

                info!("Event: {:?}", event);
        
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
                let vel = Velocity(momentum_sum/mass_sum);
                let volume = mass_sum/crate::entities::body::PLANET_DENSITY;
                let radius = crate::tools::inverse_volume_of_sphere(volume);
                let collider = Collider(Box::new(Ball::new(radius)));

                let r_com: Vector2<f32> = r_m_sum/mass_sum as f32;
                let mut transform = Transform::default();
                transform.set_translation_xyz(r_com.x, r_com.y, 0.0);
                transform.set_scale(Vector3::new(radius * crate::entities::body::PLANET_SPRITE_RATIO, radius * crate::entities::body::PLANET_SPRITE_RATIO, 1.0));


                info!("Building entity");
                // Make new entity with position of centre of mass.
                entities.build_entity()
                    .with(transform, &mut transforms)
                    .with(render.clone(), &mut sprite_renders_storage)
                    .with(vel, &mut velocities)
                    .with(collider, &mut colliders)
                    .with(Force::default(), &mut forces)
                    .with(Mass(mass_sum), &mut masses)
                    .build();
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