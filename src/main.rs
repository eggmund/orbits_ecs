#[macro_use] extern crate shrinkwraprs;
#[macro_use] extern crate log;

mod physics;
mod entities;

use amethyst::{
    core::{
        math::{Vector2, Point2},
        transform::{TransformBundle, Transform},
        Time,
    },
    prelude::*,
    renderer::{
        light::Light,
        Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        SpriteSheet, SpriteSheetFormat, Texture,
        ImageFormat,
    },
    ui::{UiFinder, UiText, UiBundle, UiCreator, RenderUi},
    input::{StringBindings, InputBundle},
    ecs::Entity,
    assets::{AssetStorage, Loader, Handle, ProgressCounter},
    utils::{application_root_dir, fps_counter},
};
use rand::rngs::ThreadRng;

const CAMERA_DIMS: (f32, f32) = (1920.0, 1080.0);

struct MainState {
    progress_counter: ProgressCounter,
    fps_display: Option<Entity>,
}

impl SimpleState for MainState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let spritesheet = self.load_spritesheet(world);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps.ron", &mut self.progress_counter);
        });

        // entities::planet::add_planet_with_rings(
        //     world,
        //     spritesheet.clone(),
        //     &mut rand_thread,
        //     Point2::new(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0),
        //     Vector2::new(0.0, 0.0),
        //     50.0,
        //     200,
        //     (20.0, 200.0),
        //     (0.5, 1.5),
        //     true,
        // );
        entities::planet::add_planet(
            world,
            spritesheet.clone(),
            Point2::new(CAMERA_DIMS.0/2.0 - 100.0, CAMERA_DIMS.1/2.0),
            Vector2::zeros(),
            30.0,
        );
        entities::planet::add_planet(
            world,
            spritesheet.clone(),
            Point2::new(CAMERA_DIMS.0/2.0 + 100.0, CAMERA_DIMS.1/2.0),
            Vector2::zeros(),
            30.0,
        );


        Self::init_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if self.fps_display.is_none() {
            info!("Setting fps display to some.");
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }

        let mut ui_text = world.write_storage::<UiText>();
        {
            if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
                if world.read_resource::<Time>().frame_number() % 20 == 0 {
                    let fps = world.read_resource::<fps_counter::FpsCounter>().sampled_fps();
                    info!("Setting fps to {}.", fps);
                    fps_display.text = format!("FPS: {:.2}", fps);
                }
            }
        }

        Trans::None
    }
}

impl MainState {
    fn new() -> Self {
        Self {
            progress_counter: ProgressCounter::default(),
            fps_display: None,
        }
    }

    fn init_camera(world: &mut World) {
        let mut transform = Transform::default();
        transform.set_translation_xyz(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0, 1.0);

        world.create_entity()
            .with(Camera::standard_2d(CAMERA_DIMS.0, CAMERA_DIMS.1))
            .with(transform)
            .build();
    }

    fn load_spritesheet(&mut self, world: &mut World) -> Handle<SpriteSheet> {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/spritesheet.png",
                ImageFormat::default(),
                &mut self.progress_counter,
                &texture_storage
            )
        };

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            "texture/spritesheet.ron",
            SpriteSheetFormat(texture_handle),
            &mut self.progress_counter,
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
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(fps_counter::FpsCounterBundle)?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(physics::GravitySystem, "gravity_system", &[])
        .with(physics::ForceSystem, "force_system", &["gravity_system"])
        .with(physics::VelocitySystem, "velocity_system", &["force_system"]);

    let mut game = Application::new(assets_dir, MainState::new(), game_data)?;
    game.run();

    Ok(())
}
