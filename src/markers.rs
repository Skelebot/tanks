use amethyst::ecs::{Component, NullStorage, DenseVecStorage};

/// Used to mark temporary entities that should be deleted when changing levels
/// Optional time after which the entity will be removed
/// Entities with a timer will still be removed on level change
#[derive(Default)]
pub struct TempMarker(pub Option<f32>);
impl Component for TempMarker {
    type Storage = DenseVecStorage<Self>;
}

/// Used to mark entities that have colliders that destroy tanks
#[derive(Default)]
pub struct DeadlyMarker;
impl Component for DeadlyMarker {
    type Storage = NullStorage<Self>;
}

#[derive(Debug)]
pub enum ColorKey {
    Background,
    Text,
    Walls,
    P1, P2, P3, P4
}
/// Used to mark entities that have a dynamic color.
pub struct DynamicColorMarker(pub ColorKey);
impl Component for DynamicColorMarker {
    type Storage = DenseVecStorage<Self>;
}