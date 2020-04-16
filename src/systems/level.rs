use nphysics2d as np;
use nalgebra as na;
use amethyst::{
    core::Transform,
    renderer::{SpriteRender},
    ecs::{
        System, Entities, Join,
        WriteStorage, Read, ReadExpect, WriteExpect,
    },
    window::ScreenDimensions,
    core::timing::Time,
    ui::UiText,
};
use crate::level::MazeLevel;
use crate::tank::{Tank, TankState};
use crate::markers::TempMarker;
use crate::utils::TanksSpriteSheet;
use crate::physics;
use crate::config::MazeConfig;
use crate::scoreboard::Scoreboard;
use crate::weapons::Weapon;

pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Read<'s, MazeConfig>,
        WriteExpect<'s, MazeLevel>,
        Entities<'s>,
        ReadExpect<'s, TanksSpriteSheet>,
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
            mut bodies,
            mut colliders,
            mut temp_markers,
            mut tanks,
            screen_dimensions,
            time,
            mut scoreboard,
            mut ui_text,
        ): Self::SystemData,
    ) {
        
        // Remove entities with a TempMarker Component (like projectiles)
        // whose timer ran out, count down timers
        for (entity, temp_marker) in (&entities, &mut temp_markers).join() {
            if let Some(ref mut timer) = temp_marker.0 {
                *timer -= time.delta_seconds();
                if *timer <= 0.0 {
                    // Remove the body and collider
                    if let Some(body) = bodies.get(entity) {
                        physics.remove_rigid_body(body.handle);
                    }
                    if let Some(collider) = colliders.get(entity) {
                        physics.remove_collider(collider.handle);
                    }
                    // Delete the entity
                    entities.delete(entity).expect("Couldn't remove the entity");
                }
            }
        }
        if let Some(ref mut timer) = level.reset_timer {
            *timer -= time.delta_seconds();

            if *timer <= 0.0 {

                // Update score for the winners
                scoreboard.update_winners(&mut ui_text);

                // Reset the level
                level.reset_timer = None;

                // Remove all entities with a TempMarker Component (like projectiles)
                for (entity, _) in (&entities, &mut temp_markers).join() {
                    // Remove bodies and colliders belonging to entities with a TempMarker Component
                    if let Some(body) = bodies.get(entity) {
                        physics.remove_rigid_body(body.handle);
                    }
                    if let Some(collider) = colliders.get(entity) {
                        physics.remove_collider(collider.handle);
                    }
                    entities.delete(entity).expect("Couldn't remove the entity");
                }

                // Reset the weapons
                for (tank, body) in (&mut tanks, &bodies).join() {
                    tank.weapon = Weapon::random();
                    tank.state = TankState::Alive;
                    // Re-enable physics bodies of all (TODO_O: Destroyed only) tanks
                    physics.get_body_mut(body.handle).unwrap().set_status(np::object::BodyStatus::Dynamic);
                }

                level.rebuild(
                    &maze_config,
                    &entities, 
                    &sprite_sheet,
                    &mut sprite_renders, 
                    &mut transforms,
                    &mut physics,
                    &mut bodies,
                    &mut colliders,
                    &mut temp_markers,
                    &screen_dimensions
                );

                // Move the tanks to new starting positions
                for (index, (_, body)) in (&tanks, &mut bodies).join().enumerate() {
                    let body = physics.get_rigid_body_mut(body.handle).unwrap();
                    body.set_position(na::Isometry2::new(
                        na::Vector2::new(level.starting_positions[index].x, level.starting_positions[index].y),
                        0.0
                    ));
                }

                physics.maintain();
            }
        }
    }
}
