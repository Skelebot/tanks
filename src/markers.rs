use amethyst::ecs::{Component, NullStorage, VecStorage, DenseVecStorage};

/// Used to mark temporary entities that should be deleted when changing levels
/// Optional time after which the entity will be removed
/// Entities with a timer will still be removed on level change
#[derive(Default)]
pub struct TempMarker(pub Option<f32>);
impl Component for TempMarker {
    type Storage = VecStorage<Self>;
}

/// Used to mark entities that have colliders that destroy tanks
#[derive(Default)]
pub struct DeadlyMarker;
impl Component for DeadlyMarker {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct AcceleratingMarker(pub f32);
impl Component for AcceleratingMarker {
    type Storage = DenseVecStorage<Self>;
}