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
        //rendy::core::vulkan::Backend,
        rendy::core::gl::Backend,
        RenderingBundle,
    },
    input::InputBundle,
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
mod input;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig {
        stdout: amethyst::StdoutLog::Colored,
        level_filter: amethyst::LogLevelFilter::Debug,
        log_file: None,
        allow_env_override: false,
        log_gfx_backend_level: None,
        log_gfx_rendy_level: None,
        module_levels: vec![],
    });

    let app_root = application_root_dir()?;
    let resources = app_root.join("res");

    let config              = resources.join("config");
    let display_config      = DisplayConfig::load(&config.join( "display.ron"))?;

    let input_bundle = InputBundle::<input::TankBindingTypes>::new()
        .with_bindings_from_file(config.join("bindings.ron")).expect("Failed to load keybindings");

    let event_loop = EventLoop::new();

    let game_data = GameDataBuilder::default()
        .with(amethyst::assets::Processor::<crate::utils::color::Colorscheme>::new(), "colorscheme_processor", &[])
        .with(systems::ColorSystem, "color_system", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<input::TankBindingTypes>::new())?
        .with_bundle(
            RenderingBundle::<Backend>::new(display_config, &event_loop)
                .with_plugin(
                    RenderToWindow::new().with_clear(ClearColor {
                        float32: [0., 0., 0., 1.0] // doesn't matter, we cover it with our custom background
                    }))
                .with_plugin(graphics::RenderFlat2D::default())
                .with_plugin(graphics::RenderShapes::default())
                .with_plugin(RenderUi::default())
        )?;

    let game = Application::build(resources, states::LoadingState::default())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            60
        )
        .build(game_data)?;
    game.run_winit_loop(event_loop);
    // Ok(())
}
