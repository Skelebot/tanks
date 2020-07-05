use amethyst::renderer::{
    batch::{GroupIterator, OneLevelBatch}, 
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    //pod::SpriteArgs,
    resources::Tint,
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    submodules::{DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub},
    types::{Backend, Texture},
    util,
};
use amethyst::assets::AssetStorage;
use amethyst::core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, SystemData, World},
    transform::Transform,
    //Hidden, HiddenPropagate,
};
use amethyst::renderer::rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso},
    hal::pso::Multisampling,
    mesh::AsVertex,
    shader::Shader,
};
use amethyst::renderer::{
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    sprite_visibility::SpriteVisibilitySortingSystem,
};
use amethyst::core::ecs::DispatcherBuilder;
use amethyst::error::Error;
extern crate derivative;
use derivative::*;
extern crate failure;
use super::pod::SpriteArgs;
use super::TintBox;

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderFlat2D {
    target: Target,
}

impl RenderFlat2D {
    /// Set target to which 2d sprites will be rendered.
    pub fn _with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderFlat2D {
    fn on_build<'a, 'b>(
        &mut self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &[],
        );
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawFlat2DDesc::new().builder())?;
            //ctx.add(
            //    RenderOrder::Transparent,
            //    DrawFlat2DTransparentDesc::new().builder(),
            //)?;
            Ok(())
        });
        Ok(())
    }
}

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2DDesc;

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawFlat2DDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, pso::CreationError> {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = FlatEnvironmentSub::new(factory).map_err(|_| pso::CreationError::Other)?;
        let textures = TextureSub::new(factory).map_err(|_| pso::CreationError::Other)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
            //vec![]
        )?;

        Ok(Box::new(DrawFlat2DCustom::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Debug)]
pub struct DrawFlat2DCustom<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, World> for DrawFlat2DCustom<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare opaque");

        let (
            sprite_sheet_storage,
            tex_storage,
            visibility,
            //_hiddens,
            //_hidden_props,
            sprite_renders,
            transforms,
            tints,
            tint_boxes,
        ) = <(
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, SpriteVisibility>,
            //ReadStorage<'_, Hidden>,
            //ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Tint>,
            ReadStorage<'_, TintBox>,
        )>::fetch(world);

        self.env.process(factory, index, world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_visibility");

            (
                &sprite_renders,
                &transforms,
                tints.maybe(),
                tint_boxes.maybe(),
                &visibility.visible_unordered,
            )
                .join()
                .filter_map(|(sprite_render, global, tint, tint_box, _)| {
                    let (batch_data, texture) = SpriteArgs::from_data(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                        tint_box
                    )?;
                    let (tex_id, _) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, world);

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            sprites_ref.prune();
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                self.sprites.data(),
            );
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw opaque");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

use amethyst::renderer::rendy::{hal::pso::ShaderStageFlags, shader::SpirvShader};

fn build_sprite_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    transparent: bool,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = SpirvShader::from_bytes(
        include_bytes!("../../res/shaders/compiled/sprite.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main"
    ).unwrap();
    let shader_vertex = unsafe { shader_vertex.module(factory).unwrap() };
    let shader_fragment = SpirvShader::from_bytes(
        include_bytes!("../../res/shaders/compiled/sprite.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main"
    ).unwrap();
    let shader_fragment = unsafe { shader_fragment.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(pso::Primitive::TriangleStrip))
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: if transparent {
                        Some(pso::BlendState::PREMULTIPLIED_ALPHA)
                    } else {
                        None
                    },
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Less,
                    write: !transparent,
                })
                // This doesn't really work for some reason
                .with_multisampling(Some(Multisampling {
                    rasterization_samples: 8,
                    sample_shading: Some(1.0),
                    sample_mask: 2,
                    alpha_coverage: true,
                    alpha_to_one: false,
                }))
                ,
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}