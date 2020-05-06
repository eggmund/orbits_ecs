pub use physics::*;

use amethyst::{
    ecs::{Component, DenseVecStorage},
    renderer::SpriteRender,
};
use crate::resources::SpriteRenders;

#[derive(Component, Debug, Copy, Clone)]
pub enum BodyType {
    Planet,
    Star,
}

impl BodyType {
    pub fn from_mass(m: f32) -> Self {
        if m > crate::entities::body::PLANET_STAR_MASS_BOUNDARY {
            BodyType::Star
        } else {
            BodyType::Planet
        }
    }

    pub fn get_render(&self, renders: &SpriteRenders) -> SpriteRender {
        match *self {
            Self::Planet | Self::Star => renders.planet.as_ref().unwrap().clone(),
        }
    }
}


pub mod physics {
    use amethyst::{
        ecs::{Component, DenseVecStorage},
        core::{
            math::{Vector2, Isometry2},
            transform::Transform,
        },
    };
    use ncollide2d::shape::Shape;
    use std::boxed::Box;

    #[derive(Shrinkwrap, Component, Copy, Clone)]
    #[shrinkwrap(mutable)]
    pub struct Velocity(pub Vector2<f32>);
    
    #[derive(Shrinkwrap, Component)]
    #[shrinkwrap(mutable)]
    pub struct Mass(pub f32);

    impl Mass {
        pub fn from_radius(r: f32, density: f32) -> Self {
            Self(crate::tools::volume_of_sphere(r) * density)
        }
    }
    
    #[derive(Shrinkwrap, Component)]
    #[shrinkwrap(mutable)]
    pub struct Force(pub Vector2<f32>); // Also acceleration. If the object has no mass, assume m = 1, so F = a

    impl Default for Force {
        fn default() -> Self {
            Self(Vector2::zeros())
        }
    }

    #[derive(Component)]
    pub struct Collider(pub Box<dyn Shape<f32>>);

    impl Collider {
        pub fn is_colliding_with(&self, this_transform: &Transform, other: &Collider, other_transform: &Transform) -> bool {
            use ncollide2d::query::{self, Proximity};

            // Get into 2D
            let (isometry1, isometry2) = {
                let translation1 = this_transform.translation();
                let vec1 = Vector2::new(translation1.x, translation1.y);
                let angle1 = this_transform.euler_angles();

                let translation2 = other_transform.translation();
                let vec2 = Vector2::new(translation2.x, translation2.y);
                let angle2 = other_transform.euler_angles();

                (Isometry2::new(vec1, angle1.0), Isometry2::new(vec2, angle2.0))
            };

            let proximity = query::proximity(
                &isometry1,
                &*(self.0), // gross
                &isometry2,
                &*(other.0),
                0.1
            );

            proximity == Proximity::Intersecting
        }
    }
}