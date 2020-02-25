extern crate serde;
use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    input::{InputBundle, StringBindings},
    utils::application_root_dir,
};
use std::time::Duration;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst_physics::PhysicsBundle;
use amethyst_nphysics::NPhysicsBackend;

mod state;
pub mod utils;
pub mod physics;
pub mod tank;
pub mod projectile;
pub mod config;
pub mod mazegen;

use config::{BulletConfig, InputConfig, TankConfig};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
let resources = app_root.join("resources");
    let display_config = resources.join("display_config.ron");
    let binding_path = resources.join("bindings.ron");
    let _config_path = resources.join("config.ron");

    let input_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(binding_path)?;

    //Configs
    let bullet_config = BulletConfig::load(&resources.join("bullet_conf.ron"));
    let input_config = InputConfig::load(&resources.join("input_conf.ron"));
    let tank_config = TankConfig::load(&resources.join("tank_conf.ron"));

    let physics_bundle = PhysicsBundle::<f32, NPhysicsBackend>::new()
        .with_pre_physics(tank::TankSystem, "tank_system".to_string(), vec![])
        .with_post_physics(utils::SpriteTransformSystem, "sp_trans_system".to_string(), vec![]);

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(physics_bundle)?
        //.with(tank::TankSystem, "tank_system", &["input_system"])
        .with(projectile::ProjectileSystem, "projectile_system", &[])
        .with(utils::SpriteTransformSystem, "physics_system", &["projectile_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([1.0, 1.0, 1.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::build(resources, state::MyState)?
        .with_resource(bullet_config)
        .with_resource(input_config)
        .with_resource(tank_config)
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
