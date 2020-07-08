use amethyst::renderer::{
    resources::Tint as TintComponent,
    rendy::core::types::vertex::AsAttribute,
    sprite::{SpriteRender, SpriteSheet},
    types::Texture,
    pod::IntoPod,
};
use amethyst::assets::{AssetStorage, Handle};
use amethyst::core::{
    math::{convert, Matrix4, Vector4},
    Transform,
};
extern crate glsl_layout;
use glsl_layout::*;
use amethyst::renderer::rendy::{
    hal::format::Format,
    mesh::{AsVertex, VertexFormat},
};
use super::TintBox as TintBoxComponent;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(16))]
pub struct Tint {
    /// Tint color as `Srgba32Sfloat`
    pub tint: vec4,
}

impl AsAttribute for Tint {
    const NAME: &'static str = "tint";
    const FORMAT: Format = Format::Rgba32Sfloat;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(16))]
pub struct TintBox {
    /// Tint color as `Srgba32Sfloat`
    pub tint: vec4,
}

impl AsAttribute for TintBox {
    const NAME: &'static str = "tint_box";
    const FORMAT: Format = Format::Rgba32Sfloat;
}
/// Sprite Vertex Data
/// ```glsl,ignore
/// vec2 dir_x;
/// vec2 dir_y;
/// vec2 pos;
/// vec2 u_offset;
/// vec2 v_offset;
/// float depth;
/// vec4 tint;
/// vec4 tint_box;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct SpriteArgs {
    /// Rotation of the sprite, X-axis
    pub dir_x: vec2,
    /// Rotation of the sprite, Y-axis
    pub dir_y: vec2,
    /// Screen position of the sprite
    pub pos: vec2,
    /// Upper-left coordinate of the sprite in the spritesheet
    pub u_offset: vec2,
    /// Bottom-right coordinate of the sprite in the spritesheet
    pub v_offset: vec2,
    /// Depth value of this sprite
    pub depth: float,
    /// Tint for this this sprite
    pub tint: vec4,
    /// The xy coordinates + width and height of a box where tint will be applied
    pub tint_box: vec4,
}

impl AsVertex for SpriteArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "dir_x"),
            (Format::Rg32Sfloat, "dir_y"),
            (Format::Rg32Sfloat, "pos"),
            (Format::Rg32Sfloat, "u_offset"),
            (Format::Rg32Sfloat, "v_offset"),
            (Format::R32Sfloat, "depth"),
            (Format::Rgba32Sfloat, "tint"),
            (Format::Rgba32Sfloat, "tint_box"),
        ))
    }
}

impl SpriteArgs {
    /// Extracts POD vertex data from the provided storages for a sprite.
    ///
    /// # Arguments
    /// * `tex_storage` - `Texture` Storage
    /// * `sprite_storage` - `SpriteSheet` Storage
    /// * `sprite_render` - `SpriteRender` component reference
    /// * `transform` - 'Transform' component reference
    pub fn from_data<'a>(
        tex_storage: &AssetStorage<Texture>,
        sprite_storage: &'a AssetStorage<SpriteSheet>,
        sprite_render: &SpriteRender,
        transform: &Transform,
        tint: Option<&TintComponent>,
        tint_box: Option<&TintBoxComponent>,
    ) -> Option<(Self, &'a Handle<Texture>)> {
        let sprite_sheet = sprite_storage.get(&sprite_render.sprite_sheet)?;
        if !tex_storage.contains(&sprite_sheet.texture) {
            return None;
        }

        let sprite = &sprite_sheet.sprites[sprite_render.sprite_number];

        let transform = convert::<_, Matrix4<f32>>(*transform.global_matrix());
        let dir_x = transform.column(0) * sprite.width;
        let dir_y = transform.column(1) * -sprite.height;
        let pos = transform * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

        Some((
            SpriteArgs {
                dir_x: dir_x.xy().into_pod(),
                dir_y: dir_y.xy().into_pod(),
                pos: pos.xy().into_pod(),
                u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
                v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
                depth: pos.z,
                tint: tint.map_or([1.0; 4].into(), |t| {
                    let (r, g, b, a) = t.0.into_components();
                    [r, g, b, a].into()
                }),
                tint_box: tint_box.map_or([-0.5, -0.5, 1.0, 1.0].into(), 
                    |t_box| vec4::from(t_box.0)
                ),
            },
            &sprite_sheet.texture,
        ))
    }
}