use amethyst::{
    assets::{Handle},
    renderer::{SpriteSheet, SpriteRender},
};

#[derive(Default, Clone)]
pub struct SpriteRenders {
    pub planet: Option<SpriteRender>,
}

