#[macro_use] extern crate shrinkwraprs;
#[macro_use] extern crate log;

mod components;
mod systems;
mod entities;
mod resources;
mod tools;
mod events;

use amethyst::{
    core::{
        math::{Vector2, Point2, Vector3},
        transform::{TransformBundle, Transform},
        Time, SystemDesc, RunNowDesc,
    },
    prelude::*,
    renderer::{
        light::Light,
        Camera,
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        SpriteSheet, SpriteSheetFormat, Texture, SpriteRender,
        ImageFormat,
    },
    ui::{UiFinder, UiText, UiBundle, UiCreator, RenderUi},
    input::{StringBindings, InputBundle},
    ecs::{Entity, Read, LazyUpdate, Entities, ReadStorage, Join, WriteStorage},
    assets::{AssetStorage, Loader, Handle, ProgressCounter, Processor},
    utils::{application_root_dir, fps_counter},
};
use rand::rngs::ThreadRng;
use rand::Rng;

const CAMERA_DIMS: (f32, f32) = (1920.0, 1080.0);

struct MainState {
    progress_counter: ProgressCounter,
    fps_display: Option<Entity>,
    sprite_sheet: Option<Handle<SpriteSheet>>,
}

impl SimpleState for MainState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let mut rand_thread = rand::thread_rng();

        Self::init_camera(world);
        self.sprite_sheet = Some(self.load_spritesheet(world));

        world.insert(resources::SpriteRenders {
            planet: Some(SpriteRender {
                sprite_sheet: self.sprite_sheet.as_ref().unwrap().clone(),
                sprite_number: 0,
            }),
        });

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps.ron", &mut self.progress_counter);
        });


        let dist = 300.0;
        self.add_planet(world, Point2::new(CAMERA_DIMS.0/2.0 - dist/2.0, CAMERA_DIMS.1/2.0 + 100.0), Vector2::zeros(), 30.0);
        self.add_planet(world, Point2::new(CAMERA_DIMS.0/2.0 - dist/2.0, CAMERA_DIMS.1/2.0 - 100.0), Vector2::zeros(), 30.0);

        self.add_planet(world, Point2::new(CAMERA_DIMS.0/2.0 + dist/2.0, CAMERA_DIMS.1/2.0 + 100.0), Vector2::zeros(), 30.0);
        self.add_planet(world, Point2::new(CAMERA_DIMS.0/2.0 + dist/2.0, CAMERA_DIMS.1/2.0 - 100.0), Vector2::zeros(), 30.0);


        // self.add_planet(
        //     world,
        //     Point2::new(CAMERA_DIMS.0/2.0, CAMERA_DIMS.1/2.0 + (dist.powi(2) - (dist/2.0).powi(2)).sqrt()), // Using pythag
        //     Vector2::zeros(),
        //     30.0
        // );
        // self.add_planet_with_rings(
        //     world,
        //     &mut rand_thread,
        //     Point2::new(CAMERA_DIMS.0, CAMERA_DIMS.1),
        //     Vector2::zeros(),
        //     50.0,
        //     10,
        //     (100.0, 250.0),
        //     (0.7, 2.0),
        //     true,
        // );
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

        {
            let mut ui_text = world.write_storage::<UiText>();

            if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
                if world.read_resource::<Time>().frame_number() % 20 == 0 {
                    let fps = world.read_resource::<fps_counter::FpsCounter>().sampled_fps();
                    fps_display.text = format!("FPS: {:.2}\n", fps);
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
            sprite_sheet: None,
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

    fn add_planet(
        &self,
        world: &mut World,
        pos: Point2<f32>,
        vel: Vector2<f32>,
        radius: f32,
    ) {
        const RAD: f32 = 30.0;
        use components::physics::*;
        use ncollide2d::shape::Ball;

        let scale = radius * entities::body::PLANET_SPRITE_RATIO;
        let scale_vec = Vector3::new(scale, scale, 0.0);

        let render = SpriteRender {
            sprite_sheet: self.sprite_sheet.as_ref().unwrap().clone(),
            sprite_number: 0,
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz(pos.x, pos.y, 0.0);
        transform.set_scale(scale_vec);

        world.create_entity()
            .with(transform)
            .with(render.clone())
            .with(Velocity(vel))
            .with(Force::default())
            .with(Mass::from_radius(RAD, entities::body::PLANET_DENSITY))
            .with(Collider(Box::new(Ball::new(RAD))))
            .build();
    }

    pub fn add_planet_with_rings(
        &self,
        world: &mut World,
        rand_thread: &mut ThreadRng,
        position: Point2<f32>,
        velocity: Vector2<f32>,
    
        main_planet_radius: f32,
        moon_num: usize,
        moon_orbit_radius_range: (f32, f32),    // Starting from surface of planet
        moon_body_radius_range: (f32, f32),
        orbit_direction_clockwise: bool,  // anticlockwise = false, clockwise = true
    ) {
        use std::f32::consts::PI;

        self.add_planet(world, position.clone(), velocity.clone(), main_planet_radius);  // Add main planet
    
        let main_planet_mass = tools::volume_of_sphere(main_planet_radius) * entities::body::PLANET_DENSITY;
        let frame_velocity = velocity;
    
        for _ in 0..moon_num {
            let orbit_radius = main_planet_radius + rand_thread.gen_range(moon_orbit_radius_range.0, moon_orbit_radius_range.1);
            let orbit_speed = tools::circular_orbit_speed(main_planet_mass, orbit_radius);
            let start_angle = rand_thread.gen_range(0.0, PI * 2.0);      // Angle from main planet to moon
            let start_pos = Point2::new(orbit_radius * start_angle.cos(), orbit_radius * start_angle.sin());   // Position on circle orbit where planet will start

            let vel_angle = if orbit_direction_clockwise {
                start_angle + PI/2.0
            } else {
                start_angle - PI/2.0
            };
            let start_velocity = Vector2::new(orbit_speed * vel_angle.cos(), orbit_speed * vel_angle.sin());
            let moon_radius = rand_thread.gen_range(moon_body_radius_range.0, moon_body_radius_range.1);
    
            self.add_planet(
                world,
                Point2::new(position.x + start_pos.x, position.y + start_pos.y),
                start_velocity + frame_velocity,  // Add velocity of main planet
                moon_radius,
            );
        }
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

        .with(systems::physics::GravitySystem, "gravity_system", &[])
        .with(systems::physics::ForceSystem, "force_system", &["gravity_system"])
        .with(systems::physics::VelocitySystem, "velocity_system", &["force_system"])
        .with(systems::physics::CollisionDetectionSystem, "collision_detection_system", &["velocity_system"])
        .with_system_desc(systems::physics::CollisionProcessingSystemDesc, "collision_processing_system", &["sprite_visibility_system", "collision_detection_system"]);

    let mut game = Application::new(assets_dir, MainState::new(), game_data)?;
    game.run();

    Ok(())
}
