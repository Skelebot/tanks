use nphysics2d as np;
use amethyst::{
    core::Transform,
    renderer::{
        resources::Tint,
    },
    ecs::{
        System, Entities, Join,
        WriteStorage, Read, ReadExpect, WriteExpect
    },
    window::ScreenDimensions,
    core::timing::Time,
    ui::UiText,
};
use crate::level::MazeLevel;
use crate::tank::{Tank, TankState};
use crate::markers::*;
use crate::physics;
use crate::config::MazeConfig;
use crate::scoreboard::Scoreboard;
use crate::weapons::Weapon;
use crate::graphics::{ShapeRender, QuadMesh};

pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'s,  MazeConfig>,
        WriteExpect<'s, MazeLevel>,
        Entities<'s>,
        ReadExpect<'s, QuadMesh>,
        WriteStorage<'s, ShapeRender>,
        WriteStorage<'s, DynamicColorMarker>,
        WriteStorage<'s, Tint>,
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
            quad_mesh,
            mut shape_renders,
            mut dyn_color_markers,
            mut tints,
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

                // Reset the weapons and tanks
                for (tank, body, tint) in (&mut tanks, &bodies, &mut tints).join() {
                    // Re-enable physics bodies of destroyed tanks
                    let rb = physics.get_rigid_body_mut(body.handle).unwrap();
                    if tank.state == TankState::Destroyed {
                        use np::object::Body;
                        rb.set_status(np::object::BodyStatus::Dynamic);
                    }
                    // Reset the velocity (this resets both angular and linear velocities)
                    rb.set_velocity(np::algebra::Velocity2::zero());

                    // Show the tank's sprite
                    tint.0.alpha = 1.0;

                    tank.weapon = Weapon::default();
                    tank.state = TankState::Alive;
                }

                level.rebuild(
                    &maze_config,
                    &entities, 
                    &quad_mesh,
                    &mut shape_renders, 
                    &mut tints,
                    &mut dyn_color_markers,
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
                    body.set_position(level.starting_positions[index]);
                }

                physics.maintain();
            }
        }
    }
}
