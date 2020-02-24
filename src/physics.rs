use amethyst::ecs::{Component, DenseVecStorage};

use amethyst_physics::objects::{PhysicsHandle, PhysicsRigidBodyTag};

pub struct Physics {
//    pub pos: na::Isometry2<f32>,
//    pub vel: na::Vector2<f32>,
    pub rb_handle: Option<PhysicsHandle<PhysicsRigidBodyTag>>,
}

impl Component for Physics {
    type Storage = DenseVecStorage<Self>;
}

/*
pub struct PhysicsSimulation {
    mech_world: DefaultMechanicalWorld<f32>,
    geom_world: DefaultGeometricalWorld<f32>,
    pub bodies: DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

impl Default for PhysicsSimulation {
    fn default() -> Self {
        PhysicsSimulation {
            mech_world: DefaultMechanicalWorld::new(na::Vector3::repeat(0.0)),
            geom_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }
}

impl PhysicsSimulation {
    #[deprecated(since = "0.0.1", note = "Use Default::default() instead")]
    pub fn new() -> Self {
        PhysicsSimulation {
            mech_world: DefaultMechanicalWorld::new(na::Vector3::repeat(0.0)),
            geom_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn add_rigid_body(&mut self, rigidbody: np::object::RigidBody<f32>) -> DefaultBodyHandle {
        self.bodies.insert(rigidbody)
    }

    pub fn add_collider(&mut self, collider: np::object::Collider<f32, DefaultBodyHandle>) -> DefaultColliderHandle {
        self.colliders.insert(collider)
    }

    #[allow(dead_code)]
    pub fn get_body(&self, handle: DefaultBodyHandle) -> Result<&dyn np::object::Body<f32>, Error> {
        match self.bodies.get(handle) {
            Some(body) => Ok(body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    #[allow(dead_code)]
    pub fn get_rigid_body(&self, handle: DefaultBodyHandle) -> Result<&np::object::RigidBody<f32>, Error> {
        match self.bodies.rigid_body(handle) {
            Some(rigid_body) => Ok(rigid_body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    #[allow(dead_code)]
    pub fn get_rigid_body_mut(&mut self, handle: DefaultBodyHandle) -> Result<&mut np::object::RigidBody<f32>, Error> {
        match self.bodies.rigid_body_mut(handle) {
            Some(rigid_body) => Ok(rigid_body),
            None => Err(Error::InvalidRBHandle { rb_handle: handle })
        }
    }

    #[allow(dead_code)]
    pub fn get_collider(&self, handle: DefaultColliderHandle) -> Result<&np::object::Collider<f32, DefaultBodyHandle>, Error> {
        match self.colliders.get(handle) {
            Some(collider) => Ok(collider),
            None => Err(Error::InvalidCLHandle { cl_handle: handle })
        }
    }

    #[allow(dead_code)]
    pub fn get_collider_mut(&mut self, handle: DefaultColliderHandle) -> Result<&mut np::object::Collider<f32, DefaultBodyHandle>, Error> {
        match self.colliders.get_mut(handle) {
            Some(collider) => Ok(collider),
            None => Err(Error::InvalidCLHandle { cl_handle: handle })
        }
    }

    pub fn step(&mut self) {
        self.mech_world.step(
            &mut self.geom_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators
        );
    }
}*/
