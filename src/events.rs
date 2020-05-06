use amethyst::{
    ecs::{Entity, Entities},
    core::{
        math::{Point2, Vector2, Vector3},
        transform::Transform,
    },
};
use std::collections::HashSet;

use crate::components::*;
use crate::systems::BodyCreationData;

#[derive(Debug)]
pub struct CollisionEvent {
    pub group: HashSet<Entity>,
}

#[derive(Debug)]
pub struct BodyCreationEvent {
    pub body_type: BodyType,
    pub position: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
    pub radius: f32,
}

impl BodyCreationEvent {
    pub fn build_entity(
        &self,
        entities: &mut Entities,
        body_system_data: &mut BodyCreationData,
    ) -> Entity {
        use ncollide2d::shape::Ball;
        use crate::entities::body::PLANET_SPRITE_RATIO;

        let scale = PLANET_SPRITE_RATIO * self.radius;
        let mut transform = Transform::default();
        transform.set_translation_xyz(self.position.x, self.position.y, 0.0);
        transform.set_scale(Vector3::new(scale, scale, 1.0));

        let render = self.body_type.get_render(&(*body_system_data.renders_resource));

        entities.build_entity()
            .with(render, &mut body_system_data.render_storage)
            .with(self.body_type, &mut body_system_data.body_type)
            .with(transform, &mut body_system_data.transforms)
            .with(Velocity(self.velocity), &mut body_system_data.velocities)
            .with(Mass(self.mass), &mut body_system_data.masses)
            .with(Force::default(), &mut body_system_data.forces)
            .with(Collider(Box::new(Ball::new(self.radius))), &mut body_system_data.colliders)
            .build()
    }
}