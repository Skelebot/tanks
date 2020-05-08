extern crate nphysics2d;
extern crate ncollide2d;
extern crate nalgebra;
use amethyst::{
    core::transform::TransformBundle,
    core::frame_limiter::FrameRateLimitStrategy,
    prelude::*,
    renderer::{
        plugins::RenderToWindow,
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    input::{InputBundle, StringBindings},
    utils::application_root_dir,
    ui::{RenderUi, UiBundle},
    window::{DisplayConfig, EventLoop},
};
use std::time::Duration;

mod states;
mod level;
mod utils;
mod config;
mod systems;
mod markers;
mod tank;
mod scoreboard;
mod physics;
mod weapons;
mod graphics;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let resources = app_root.join("res");

    let config      = resources.join("config");
    let display_config      = DisplayConfig::load(&config.join( "display.ron"))?;
    let tank_config         = config::TankConfig::load(&config.join(    "tank.ron")).unwrap();
    let maze_config         = config::MazeConfig::load(&config.join(    "maze.ron")).unwrap();
    let beamer_config       = config::BeamerConfig::load(&config.join(  "beamer.ron")).unwrap();
    let cannon_config       = config::CannonConfig::load(&config.join(  "cannon.ron")).unwrap();
    let spawn_config        = config::SpawnConfig::load(&config.join(   "spawn.ron")).unwrap();
    let destroy_config      = config::DestroyConfig::load(&config.join( "destroy.ron")).unwrap();
    let performance_config  = config::PerformanceConfig::load(&config.join( "performance.ron")).unwrap();

    let input_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(config.join("bindings.ron")).expect("Failed to load keybindings");

    let event_loop = EventLoop::new();

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(systems::LevelSystem, "level_system", &[])
        .with(systems::TankSystem, "tank_system", &["input_system", "level_system"])
        .with(systems::SpawnSystem::default(), "spawn_system", &["level_system"])

        .with(systems::BeamerSystem, "beamer_system", &["spawn_system"])
        .with(systems::CannonSystem, "cannon_system", &["spawn_system"])
        // .with(systems::RocketSystem, "rocket_system", &["spawn_system"])

        .with(systems::DestroySystem, "destroy_system", &["beamer_system", "cannon_system"])
        .with(systems::CameraShakeSystem, "shake_system", &["destroy_system"])
        .with(physics::StepperSystem, "stepper_system", &["destroy_system"])
        .with(physics::PTTSystem, "physics_to_transform_system", &["stepper_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                .with_plugin(
                    RenderToWindow::new().with_clear(ClearColor {
                        // float32: [0.918, 0.918, 0.918, 1.0]    //light background
                        float32: [0.0145, 0.0165, 0.0204, 1.0]   //dark background
                    }))
                .with_plugin(graphics::RenderFlat2D::default())
                .with_plugin(RenderUi::default())
        )?;

    let game = Application::build(resources, states::GameplayState)?
        .with_resource(tank_config)
        .with_resource(maze_config)
        .with_resource(beamer_config)
        .with_resource(cannon_config)
        .with_resource(spawn_config)
        .with_resource(destroy_config)
        .with_resource(performance_config)
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60
        )
        .build(game_data)?;
    game.run_winit_loop(event_loop);
    // Ok(())
}
