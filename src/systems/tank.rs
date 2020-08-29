use nphysics2d as np;
use nalgebra as na;
use amethyst::{
    core::{
        timing::Time,
    },
    ecs::{
        System, Join,
        Read, ReadExpect, ReadStorage, WriteExpect, WriteStorage,
    },
    input::InputHandler,
};
use crate::tank::{Tank, TankState};
use crate::physics;
use crate::config::TankConfig;
use crate::config::BeamerConfig;
use crate::weapons::Weapon;
use crate::input::*;

pub struct TankSystem;

impl<'s> System<'s> for TankSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Tank>,
        Read<'s, InputHandler<TankBindingTypes>>,
        ReadExpect<'s,  TankConfig>,
        ReadStorage<'s, physics::Body>,
        WriteExpect<'s, physics::Physics>,
        Read<'s, Time>,

        ReadExpect<'s, BeamerConfig>
    );

    fn run(
        &mut self,
        (
            mut tanks,
            input,
            tank_config,
            bodies,
            mut physics,
            time,
            beamer_config
        ): Self::SystemData,
    ) {
        for (tank, body) in (&mut tanks, &bodies).join() {
            // Do not control dead tanks
            if tank.state == TankState::Alive {
                let (mov_forward, mov_side, fire, ability) = (
                    input.axis_value(&AxisBinding::Throttle(tank.team.into())).expect("axis Throttle not defined"),
                    input.axis_value(&AxisBinding::Steering(tank.team.into())).expect("axis Steering not defined"),
                    input.action_is_down(&ActionBinding::Shoot(tank.team.into())).expect("action Shoot not defined"),
                    input.action_is_down(&ActionBinding::Ability(tank.team.into())).expect("action Ability not defined"),
                );

                tank.is_shooting = fire;
                tank.is_using_ability = ability;

                let mut lock_rotation = false;
                let mut lock_movement = false;

                // We want the player to be unable to move when shooting a laser beam
                if let Weapon::Beamer {
                    ref mut overheat_timer,
                    ref mut shooting_timer,
                    ..
                } = tank.weapon {
                    if tank.is_shooting && overheat_timer.is_none() && shooting_timer.is_none() {
                        // The tank is heating up the weapon
                        lock_rotation = beamer_config.lock_rotation_when_heating;
                        lock_movement = beamer_config.lock_movement_when_heating;
                    } else if shooting_timer.is_some() {
                        // The tank is shooting
                        lock_rotation = beamer_config.lock_rotation_when_shooting;
                        lock_movement = beamer_config.lock_movement_when_shooting;
                    }
                }

                if tank.state == TankState::Stunned {
                    tank.is_shooting = false;
                    tank.is_using_ability = false;
                    lock_movement = true;
                    lock_rotation = true;
                }

                let rb = physics.get_rigid_body_mut(body.handle).unwrap();

                // Movement rotated relative to the tank's front and scaled by delta time
                let movement = na::Vector2::new(
                    if !lock_rotation { mov_side } else { 0.0 }, 
                    if !lock_movement { mov_forward } else { 0.0 },
                );

                let mov_rel = rb.position().rotation * na::Vector2::new(0.0, movement.y * tank_config.linear_accel * time.delta_seconds() * 100.0);

                // Push the tank forward and apply angular velocity
                rb.set_velocity(
                    *rb.velocity() +
                    np::math::Velocity::new(
                        na::Vector2::new(mov_rel.x, mov_rel.y),
                        -movement.x * tank_config.angular_accel * time.delta_seconds() * 100.0
                    )
                );
            }
        }
    }
}
