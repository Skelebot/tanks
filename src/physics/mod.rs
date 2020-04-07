use nphysics2d as np;
use nalgebra as na;

use self::np::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};
use self::np::object::{DefaultBodySet, DefaultColliderSet, DefaultBodyHandle, DefaultColliderHandle};
use self::np::joint::DefaultJointConstraintSet;
use self::np::force_generator::DefaultForceGeneratorSet;
use amethyst::ecs::{Component, DenseVecStorage};

mod systems;
pub use systems::*;

pub struct Collider { pub handle: DefaultColliderHandle }
impl Component for Collider { type Storage = DenseVecStorage<Self>; }
impl Collider {
    pub fn new(handle: DefaultColliderHandle) -> Self {
        Collider { handle }
    }
}
pub struct Body { pub handle: DefaultBodyHandle }
impl Component for Body { type Storage = DenseVecStorage<Self>; }
impl Body {
    pub fn new(handle: DefaultBodyHandle) -> Self {
        Body { handle }
    }
}

pub struct Physics {
    pub mech_world: DefaultMechanicalWorld<f32>,
    pub geom_world: DefaultGeometricalWorld<f32>,
    pub bodies: DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    _joint_constraints: DefaultJointConstraintSet<f32>,
    _force_generators: DefaultForceGeneratorSet<f32>,
}

impl Physics {
    pub fn new() -> Self {
        let mut mech_world = DefaultMechanicalWorld::new(na::Vector2::new(0.0, 0.0));
        mech_world.solver.set_contact_model(Box::new(np::solver::SignoriniModel::new()));
        Physics {
            mech_world,
            geom_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            _joint_constraints: DefaultJointConstraintSet::new(),
            _force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn add_rigid_body(&mut self, rigidbody: np::object::RigidBody<f32>) -> DefaultBodyHandle {
        self.bodies.insert(rigidbody)
    }

    pub fn add_collider(&mut self, collider: np::object::Collider<f32, DefaultBodyHandle>) -> DefaultColliderHandle {
        self.colliders.insert(collider)
    }

    pub fn remove_rigid_body(&mut self, rb_handle: np::object::DefaultBodyHandle) {
        self.bodies.remove(rb_handle);
    }

    pub fn remove_collider(&mut self, col_handle: np::object::DefaultColliderHandle) {
        self.colliders.remove(col_handle);
    }

    #[allow(dead_code)]
    pub fn get_body(&self, handle: DefaultBodyHandle) -> Option<&dyn np::object::Body<f32>> {
        match self.bodies.get(handle) {
            Some(body) => Some(body),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_body_mut(&mut self, handle: DefaultBodyHandle) -> Option<&mut dyn np::object::Body<f32>> {
        match self.bodies.get_mut(handle) {
            Some(body) => Some(body),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_rigid_body(&self, handle: DefaultBodyHandle) -> Option<&np::object::RigidBody<f32>> {
        match self.bodies.rigid_body(handle) {
            Some(rigid_body) => Some(rigid_body),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_rigid_body_mut(&mut self, handle: DefaultBodyHandle) -> Option<&mut np::object::RigidBody<f32>> {
        match self.bodies.rigid_body_mut(handle) {
            Some(rigid_body) => Some(rigid_body),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_collider(&self, handle: DefaultColliderHandle) -> Option<&np::object::Collider<f32, DefaultBodyHandle>> {
        match self.colliders.get(handle) {
            Some(collider) => Some(collider),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_collider_mut(&mut self, handle: DefaultColliderHandle) -> Option<&mut np::object::Collider<f32, DefaultBodyHandle>> {
        match self.colliders.get_mut(handle) {
            Some(collider) => Some(collider),
            None => None,
        }
    }

    pub fn maintain(&mut self) {
        self.mech_world.maintain(&mut self.geom_world, &mut self.bodies, &mut self.colliders, &mut self._joint_constraints);
        self.geom_world.maintain(&mut self.bodies, &mut self.colliders);
    }

    pub fn step(&mut self) {
        self.mech_world.step(
            &mut self.geom_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self._joint_constraints,
            &mut self._force_generators
        );
    }
}
