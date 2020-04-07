use amethyst::{
    core::Transform,
    renderer::{SpriteRender},
    ecs::{
        System, Entities, 
        WriteStorage, Read, ReadExpect, WriteExpect,
    },
    window::ScreenDimensions,
    core::timing::Time,
    ui::UiText,
};
use crate::level::MazeLevel;
use crate::tank::Tank;
use crate::markers::TempMarker;
use crate::utils::SpriteSheetRes;
use crate::physics;
use crate::config::MazeConfig;
use crate::scoreboard::Scoreboard;

pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Read<'s, MazeConfig>,
        WriteExpect<'s, MazeLevel>,
        Entities<'s>,
        Read<'s, SpriteSheetRes>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, Tank>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, Time>,

        WriteExpect<'s, Scoreboard>,
        WriteStorage<'s, UiText>,
    );

    fn run(
        &mut self,
        (
            maze_config,
            mut level,
            entities,
            sprite_sheet,
            mut sprite_renders,
            mut transforms,
            mut physics,
            bodies,
            colliders,
            temp_markers,
            mut tanks,
            screen_dimensions,
            time,
            mut scoreboard,
            mut ui_text,
        ): Self::SystemData,
    ) {
        if let Some(ref mut timer) = level.reset_timer {
            *timer -= time.delta_seconds();
            if *timer <= 0.0 {
                // Update score for the winners
                scoreboard.update_winners(&mut ui_text);
                // Reset the level
                level.reset_timer = None;
                level.reset_level(
                    &maze_config,
                    &entities, 
                    &sprite_sheet,
                    &mut sprite_renders, 
                    &mut transforms,
                    &mut physics,
                    bodies,
                    colliders,
                    &screen_dimensions,
                    temp_markers,
                    &mut tanks
                );
                physics.maintain();
            }
        }
    }
}
