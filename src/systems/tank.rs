use amethyst::{
    core::{
        math as na,
        timing::Time,
    },
    ecs::{
        System, Read, ReadStorage, WriteStorage, Join
    },
    input::{InputHandler, StringBindings},
};
use specs_physics::{
    nphysics::object::RigidBody,
    nphysics::algebra::Velocity2,
    BodyComponent,
};
use crate::tank::{Tank, Team};
use crate::config::TankConfig;

pub struct TankSystem;

impl<'s> System<'s> for TankSystem {
    type SystemData = (
        ReadStorage<'s, Tank>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, TankConfig>,
        WriteStorage<'s, BodyComponent<f32>>,
        Read<'s, Time>
    );

    fn run(
        &mut self,
        (
            tanks,
            input,
            tank_config,
            mut bodies,
            _time
        ): Self::SystemData,
    ) {
        for (tank, body) in (&tanks, &mut bodies).join() {
            let body = body.downcast_mut::<RigidBody<f32>>().unwrap();
            // TODO: Parametric input axis names and teams for any arbitrary number of players
            let (mov_forward, mov_side) = match tank.team {
                Team::Red => (
                    input.axis_value("p1_forward"),
                    input.axis_value("p1_side"),
                ),
                Team::Blue => (
                    input.axis_value("p2_forward"),
                    input.axis_value("p2_side"),
                )
            };

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

            // Movement rotated relative to the tank's front
            let mov_rel = body.position().rotation
                * na::Vector2::new(0.0, movement.y * tank_config.linear_accel);

            // TODO: Delta frame time scaling
            // Push the tank forward and apply angular velocity
            body.set_velocity(
                * body.velocity() +
                Velocity2::new(
                    na::Vector2::new(mov_rel.x, mov_rel.y),
                    movement.x * tank_config.angular_accel
                )
            );
        }
    }
}