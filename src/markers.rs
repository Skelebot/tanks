use amethyst::ecs::{Component, NullStorage};

/// Used to mark temporary entities that should be deleted when changing levels
#[derive(Default)]
pub struct TempMarker;
impl Component for TempMarker {
    type Storage = NullStorage<Self>;
}

/// Used to mark entities that have sensors that destroy tanks
#[derive(Default)]
pub struct Beam;
impl Component for Beam {
    type Storage = NullStorage<Self>;
}