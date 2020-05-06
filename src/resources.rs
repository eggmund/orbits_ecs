use amethyst::{
    assets::{Handle},
    renderer::{SpriteSheet, SpriteRender},
    core::{
        math::{Vector2},
    },
};

#[derive(Default, Clone)]
pub struct SpriteRenders {
    pub planet: Option<SpriteRender>,
}

#[derive(Default, Clone)]
pub struct MouseInfo {
    pub click_pos: Option<Vector2<f32>>,
    pub is_down: bool,
}