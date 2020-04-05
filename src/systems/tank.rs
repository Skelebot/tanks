use nphysics2d as np;
use nalgebra as na;
use amethyst::{
    core::{
        timing::Time,
    },
    ecs::{
        System, Read, ReadStorage, WriteExpect, WriteStorage, Join
    },
    input::{InputHandler, StringBindings},
};
use crate::tank::{Tank, Team};
use crate::physics;
use crate::config::TankConfig;

pub struct TankSystem;

impl<'s> System<'s> for TankSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Tank>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, TankConfig>,
        ReadStorage<'s, physics::Body>,
        WriteExpect<'s, physics::Physics>,
        Read<'s, Time>
    );

    fn run(
        &mut self,
        (
            mut tanks,
            input,
            tank_config,
            bodies,
            mut physics,
            time
        ): Self::SystemData,
    ) {
        for (tank, body) in (&mut tanks, &bodies).join() {
            // TODO_L: Parametric input axis names and teams for any arbitrary number of players
            let (mov_forward, mov_side, fire) = match tank.team {
                Team::Red => (
                    input.axis_value("p1_forward"),
                    input.axis_value("p1_side"),
                    input.action_is_down("p1_fire")
                ),
                Team::Blue => (
                    input.axis_value("p2_forward"),
                    input.axis_value("p2_side"),
                    input.action_is_down("p2_fire")
                )
            };

            tank.is_shooting = fire.expect("Something went wrong reading input");

            // Change tank velocity
            let mut movement = na::Vector2::repeat(0.0);
            // Check if there is forward/backward movement
            if let Some(fwd) = mov_forward {
                movement.y += fwd;
            }
            // Check for right/left (side) movement
            if let Some(side) = mov_side {
                movement.x += side;
            }

            let rb = physics.get_rigid_body_mut(body.handle).unwrap();

            // Movement rotated relative to the tank's front
            let mov_rel = rb.position().rotation * na::Vector2::new(0.0, movement.y * tank_config.linear_accel * time.delta_seconds() * 100.0);

            // TODO: Delta frame time scaling
            // Push the tank forward and apply angular velocity
            rb.set_velocity(
                * rb.velocity() +
                np::math::Velocity::new(
                    na::Vector2::new(mov_rel.x, mov_rel.y),
                    -movement.x * tank_config.angular_accel * time.delta_seconds() * 100.0
                )
            );
        }
    }
}
