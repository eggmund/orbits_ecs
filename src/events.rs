use amethyst::{
    ecs::Entity,
};

use std::collections::HashSet;

#[derive(Debug)]
pub struct CollisionEvent {
    pub group: HashSet<Entity>,
}