use amethyst::{
    core::transform::{TransformBundle, Transform},
    core::frame_limiter::FrameRateLimitStrategy,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    input::{InputBundle, StringBindings},
    utils::application_root_dir,
};
use specs_physics::{
    nalgebra as na,
    PhysicsBundle
};
use std::time::Duration;

mod states;
mod level;
mod utils;
mod config;
mod tank;
mod systems;
mod markers;
fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let resources = app_root.join("res");

    let config = resources.join("config");
    let display_config_path = config.join("display.ron");
    let tank_config = config::TankConfig::load(&config.join("tank.ron")).expect("Failed to load TankConfig");

    let physics_bundle = PhysicsBundle::<f32, Transform>::new(
        na::Vector::y() * 0.0,
         &["level_system"]
    ).with_fixed_stepper(30);

    let input_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(config.join("bindings.ron")).expect("Failed to load keybindings");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(systems::TankSystem, "tank_system", &["input_system"])
        .with(systems::LevelSystem, "level_system", &["tank_system"])
        .with_bundle(physics_bundle)?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::build(resources, states::GameplayState{maze_r: false})?
        .with_resource(tank_config)
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
