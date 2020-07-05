pub mod mazegen;
pub mod color;

use amethyst::assets::Handle;
use amethyst::renderer::SpriteSheet;

#[derive(Clone)]
pub struct TanksSpriteSheet {
    pub handle: Handle<SpriteSheet>,
}
impl TanksSpriteSheet {
    pub fn new(handle: Handle<SpriteSheet>) -> Self {
        Self { handle }
    }
}

#[derive(Clone)]
pub struct SpawnsSpriteSheet {
    pub handle: Handle<SpriteSheet>,
}
impl SpawnsSpriteSheet {
    pub fn new(handle: Handle<SpriteSheet>) -> Self {
        Self { handle }
    }
}