#[macro_use] extern crate shrinkwraprs;

mod physics;

use amethyst::{
    core::{
        math::{Vector2, Vector3, Point2},
        transform::{TransformBundle, Transform}
    },
    prelude::*,
    renderer::{
        Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
        ImageFormat,
    },
    assets::{AssetStorage, Loader, Handle},
    utils::application_root_dir,
};
use std::f32::consts::PI;

const CAMERA_DIMS: (f32, f32) = (800.0, 600.0);

struct MainState;

impl SimpleState for MainState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let spritesheet = Self::load_spritesheet(world);

        Self::add_planet(
            world,
            spritesheet.clone(),
            Point2::new(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0),
            Vector2::new(0.0, 0.0),
            10.0,
        );

        Self::add_planet(
            world,
            spritesheet,
            Point2::new(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/3.0),
            Vector2::new(0.0, 0.0),
            10.0,
        );


        Self::init_camera(world);
    }
}

impl MainState {
    fn init_camera(world: &mut World) {
        let mut transform = Transform::default();
        transform.set_translation_xyz(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0, 1.0);

        world.create_entity()
            .with(Camera::standard_2d(CAMERA_DIMS.0, CAMERA_DIMS.1))
            .with(transform)
            .build();
    }

    fn load_spritesheet(world: &mut World) -> Handle<SpriteSheet> {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/planet_spritesheet.png",
                ImageFormat::default(),
                (),
                &texture_storage
            )
        };

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            "texture/planet_spritesheet.ron",
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_store,
        )
    }

    fn add_planet(
        world: &mut World,
        sprite_sheet_handle: Handle<SpriteSheet>,
        pos: Point2<f32>,
        vel: Vector2<f32>,
        radius: f32
    ) {
        const SPRITE_RADIUS: f32 = 32.0/2.0;    // Radius of default sprite = width/2.0
        const SPRITE_RATIO: f32 = 1.0/SPRITE_RADIUS;
        const PLANET_DENSITY: f32 = 5000.0;

        // Get the planet sprite from the spritesheet
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 0,
        };

        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(SPRITE_RATIO * radius, SPRITE_RATIO * radius, 0.0));
        transform.set_translation_xyz(pos.x, pos.y, 0.0);

        // m = v * density
        let mass = 4.0/3.0 * PI * radius.powi(3) * PLANET_DENSITY;

        world.create_entity()
            .with(sprite_render)
            .with(transform)
            .with(physics::Velocity(vel))
            .with(physics::Mass(mass))
            .with(physics::Force(Vector2::zeros()))
            .build();
    }
}


fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(TransformBundle::new())?
        .with(physics::GravitySystem, "gravity_system", &[])
        .with(physics::ForceSystem, "force_system", &["gravity_system"])
        .with(physics::VelocitySystem, "velocity_system", &["force_system"]);

    let mut game = Application::new(assets_dir, MainState, game_data)?;
    game.run();

    Ok(())
}
