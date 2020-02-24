use amethyst::{
    core::math as na,
    core::timing::Time, //Delta time
    ecs::{Component, DenseVecStorage, Join, Read, ReadExpect, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
};

use crate::config::InputConfig;
use crate::physics::Physics;

use amethyst_physics::prelude::*;

pub const TANK_SIZE: f32 = 6.0;

#[derive(PartialEq, Eq, Debug)]
pub enum Team {
    Red,
    Blue,
}

pub struct Tank {
    pub team: Team,
    pub size: f32,
    //shoot timeout
    //weapon
    //ammo
}

impl Tank {
    pub fn new(team: Team) -> Self {
        Tank {
            team,
            size: TANK_SIZE,
        }
    }
}

impl Component for Tank {
    type Storage = DenseVecStorage<Self>;
}

pub struct TankSystem;

//TODO: Move sprite Transform logic to PhysicsSystem
impl<'s> System<'s> for TankSystem {
    type SystemData = (
        WriteStorage<'s, Physics>,
        ReadStorage<'s, Tank>,
        ReadExpect<'s, PhysicsWorld<f32>>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, InputConfig>,
        Read<'s, Time>, //Delta time
    );

    fn run(
        &mut self,
        (mut physics, tanks, phys_world, input, _input_config, _time): Self::SystemData,
    ) {
        for (tank, phys) in (&tanks, &mut physics).join() {
            //TODO: Parametric input &str-s for arbitrary number of players
            let (mov_forward, mov_side, fire) = match tank.team {
                Team::Red => (
                    input.axis_value("p1_forward"),
                    input.axis_value("p1_side"),
                    input.action_is_down("p1_fire"),
                ),
                Team::Blue => (
                    input.axis_value("p2_forward"),
                    input.axis_value("p2_side"),
                    input.action_is_down("p2_fire"),
                ),
            };

            //Change tank velocity
            let mut movement = na::Vector2::repeat(0.0);
            //Check if there is forward/backward movement
            if let Some(fwd) = mov_forward {
                movement.y += fwd;
            }
            //Check if there is right/left movement
            if let Some(side) = mov_side {
                movement.x += side;
            }

            let rb_handle = phys.rb_handle.as_ref().unwrap().get();

            //Movement relative to the tank's front
            let mov_rel = phys_world.rigid_body_server().transform(rb_handle).rotation
                * na::Vector3::new(0.0, movement.y * 300.0, 0.0);

            //TODO: Delta
            //Move the tank forward and backward
            phys_world
                .rigid_body_server()
                .apply_impulse(rb_handle, &mov_rel);

            //Rotate the tank
            //FIXME: For some reason apply_angular_impulse does not work
            phys_world.rigid_body_server().set_angular_velocity(
                rb_handle,
                &(phys_world.rigid_body_server().angular_velocity(rb_handle)
                    + na::Vector3::new(0.0, 0.0, movement.x * 0.1)),
            );

            if movement.y == 0.0 {
                //Apply friction ourselves
                phys_world.rigid_body_server().set_linear_velocity(
                    rb_handle,
                    &(to_zero(
                        &phys_world.rigid_body_server().linear_velocity(rb_handle),
                        2.0, //We can't use this because it's unimplemented!() XDD
                             //So we have to stick to a constant
                             //phys_world.rigid_body_server().friction(rb_handle)
                    )),
                );
            }
            if movement.x == 0.0 {
                phys_world.rigid_body_server().set_angular_velocity(
                    rb_handle,
                    &(to_zero(
                        &phys_world.rigid_body_server().angular_velocity(rb_handle),
                        1.0,
                    )),
                );
            }

            //println!("{:?}", phys_world.rigid_body_server().transform(rb_handle));

            //Shooting recoil (broken)
            if let Some(shoot) = fire {
                if shoot {
                    let recoil_vec = phys_world.rigid_body_server().transform(rb_handle).rotation
                        * na::Vector3::new(0.0, -movement.y * 1000.0, 0.0);
                    println!("{:?}", recoil_vec);
                    phys_world
                        .rigid_body_server()
                        .apply_impulse(rb_handle, &recoil_vec);
                }
            }
        }
    }
}

//Brings the vector3 closer to zero by specified amount
fn to_zero(vec: &na::Vector3<f32>, amount: f32) -> na::Vector3<f32> {
    let x = if vec.x > 0.0 {
        0.0_f32.max(vec.x - amount)
    } else if vec.x < 0.0 {
        0.0_f32.min(vec.x + amount)
    } else {
        0.0_f32
    };
    let y = if vec.y > 0.0 {
        0.0_f32.max(vec.y - amount)
    } else if vec.y < 0.0 {
        0.0_f32.min(vec.y + amount)
    } else {
        0.0_f32
    };
    let z = if vec.z > 0.0 {
        0.0_f32.max(vec.z - amount)
    } else if vec.z < 0.0 {
        0.0_f32.min(vec.z + amount)
    } else {
        0.0_f32
    };
    return na::Vector3::new(x, y, z);
}
