extern crate nphysics2d;
extern crate ncollide2d;
extern crate nalgebra;
use amethyst::{
    core::transform::TransformBundle,
    core::frame_limiter::FrameRateLimitStrategy,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    input::{InputBundle, StringBindings},
    utils::application_root_dir,
    ui::{RenderUi, UiBundle},
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

    let config = resources.join("config");
    let display_config_path = config.join("display.ron");
    let tank_config = config::TankConfig::load(&config.join("tank.ron")).expect("Failed to load TankConfig");
    let maze_config = config::MazeConfig::load(&config.join("maze.ron")).expect("Failed to load MazeConfig");

    let input_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(config.join("bindings.ron")).expect("Failed to load keybindings");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(systems::TankSystem, "tank_system", &["input_system"])
        .with(systems::LevelSystem, "level_system", &["tank_system"])

        .with(systems::BeamerSystem, "beamer_system", &["level_system"])
        .with(systems::CannonSystem, "cannon_system", &["level_system"])

        .with(systems::DestroySystem, "destroy_system", &["beamer_system", "cannon_system"])
        .with(physics::StepperSystem, "stepper_system", &["destroy_system"])
        .with(physics::PTTSystem, "physics_to_transform_system", &["stepper_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        //.with_clear([0.918, 0.918, 0.918])    //light background
                        .with_clear([0.0145, 0.0165, 0.0204])   //dark background
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
        )?;

    let mut game = Application::build(resources, states::GameplayState)?
        .with_resource(tank_config)
        .with_resource(maze_config)
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
