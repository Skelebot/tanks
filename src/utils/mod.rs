pub mod mazegen;

use amethyst::assets::Handle;
use amethyst::renderer::SpriteSheet;
#[derive(Default)]
pub struct SpriteSheetRes {
    pub handle: Option<Handle<SpriteSheet>>,
}