#[macro_use] extern crate shrinkwraprs;

mod physics;
mod entities;

use amethyst::{
    core::{
        math::{Vector2, Point2},
        transform::{TransformBundle, Transform}
    },
    prelude::*,
    renderer::{
        Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        SpriteSheet, SpriteSheetFormat, Texture,
        ImageFormat,
    },
    assets::{AssetStorage, Loader, Handle},
    utils::application_root_dir,
};

const CAMERA_DIMS: (f32, f32) = (800.0, 600.0);

struct MainState;

impl SimpleState for MainState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let spritesheet = Self::load_spritesheet(world);
        let mut rand_thread = rand::thread_rng();

        entities::planet::add_planet_with_rings(
            world,
            spritesheet.clone(),
            &mut rand_thread,
            Point2::new(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0),
            Vector2::new(0.0, 0.0),
            50.0,
            1000,
            (20.0, 200.0),
            (0.5, 1.5),
            true,
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
