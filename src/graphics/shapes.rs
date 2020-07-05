use amethyst::renderer::{
    batch::{GroupIterator, OneLevelBatch}, 
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    resources::Tint,
    pod::ViewArgs,
    submodules::{DynamicVertexBuffer, DynamicUniform, gather::CameraGatherer},
    types::{Backend, Mesh},
    util,
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    pod::VertexArgs,
};
use amethyst::assets::AssetStorage;
use amethyst::core::{
    ecs::{Join, Read, ReadStorage, SystemData, World, Component, DenseVecStorage},
    transform::Transform,
};
use amethyst::renderer::rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    core::types::vertex::Position,
    hal::{self, device::Device, pso},
    mesh::AsVertex,
    shader::Shader,
};
use amethyst::assets::Handle;
use amethyst::error::Error;
extern crate derivative;
use derivative::*;
extern crate failure;

pub struct ShapeRender {
    pub mesh: Handle<Mesh>,
}
impl Component for ShapeRender {
    type Storage = DenseVecStorage<Self>;
}

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderShapes {
    target: Target,
}

impl RenderShapes {
    /// Set target to which 2d sprites will be rendered.
    pub fn _with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

// TODO: Remove this, because we removed the transparent sprite drawing logic
impl<B: Backend> RenderPlugin<B> for RenderShapes {
    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawShapeDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawShapeDesc;

impl DrawShapeDesc {
    /// Create instance of `DrawShape` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawShapeDesc {
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

        //let env = FlatEnvironmentSub::new(factory).map_err(|_| pso::CreationError::Other)?;

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let vertex = DynamicVertexBuffer::new();

        log::info!("Start creating pipeline");

        let (pipeline, pipeline_layout) = build_shapes_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout()],
        )?;

        log::info!("Finished creating pipeline");

        Ok(Box::new(DrawShapeCustom::<B> {
            pipeline,
            pipeline_layout,
            env,
            vertex,
            shapes: Default::default(),
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Debug)]
pub struct DrawShapeCustom<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, ViewArgs>,
    vertex: DynamicVertexBuffer<B, VertexArgs>,
    shapes: OneLevelBatch<u32, VertexArgs>,
}

impl<B: Backend> RenderGroup<B, World> for DrawShapeCustom<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");

        let (
            shape_renders,
            transforms,
            tints,
            mesh_storage,
        ) = <(
            ReadStorage<'_, ShapeRender>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Tint>,
            Read<'_, AssetStorage<Mesh>>,
        )>::fetch(world);

        let cam = CameraGatherer::gather(world);
        self.env.write(factory, index, cam.projview);
        self.shapes.clear_inner();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_meshes");
            
            (&shape_renders, &transforms, tints.maybe())
            .join()
            .map(|(sprite_render, transform, tint)| {
                let batch_data = VertexArgs::from_object_data(transform, tint);
                let mesh_id = sprite_render.mesh.id();
                (mesh_id, batch_data)
            })
            .for_each_group(|mesh_id, batch_data| {
                if mesh_storage.contains_id(mesh_id) {
                    self.shapes.insert(mesh_id, batch_data.drain(..))
                }
            });
        }

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            self.shapes.prune();
            self.vertex.write(
                factory,
                index,
                self.shapes.count() as u64,
                self.shapes.data(),
            );
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw opaque");
        
        let mesh_storage = <Read<'_, AssetStorage<Mesh>>>::fetch(world);

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);

        let meshes_loc = 1; // vec![Position::vertex()].len();
        self.vertex.bind(index, meshes_loc, 0, &mut encoder);

        for (&mesh_id, range) in self.shapes.iter() {
            if let Some(mesh) = B::unwrap_mesh(unsafe { mesh_storage.get_by_id_unchecked(mesh_id)})
            {
                // These two do the same thing, but the second isn't unsafe and dumps errors
                //mesh.bind(0, &[Position::vertex()], &mut encoder).unwrap();
                //unsafe {
                //    encoder.draw(0..mesh.len(), 0..1);
                //}
                mesh.bind_and_draw(
                    0,
                    &vec![Position::vertex()],
                    range,
                    &mut encoder,
                ).unwrap();
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

fn build_shapes_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {

    log::info!("Creating pipeline layout");

    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    log::info!("Creating shaders");

    let shader_vertex = SpirvShader::from_bytes(
        include_bytes!("../../res/shaders/compiled/shapes.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main"
    ).unwrap();
    let shader_vertex = unsafe { shader_vertex.module(factory).unwrap() };
    let shader_fragment = SpirvShader::from_bytes(
        include_bytes!("../../res/shaders/compiled/shapes.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main"
    ).unwrap();
    let shader_fragment = unsafe { shader_fragment.module(factory).unwrap() };

    //let vertex_desc = vec![Position::vertex()].iter()
    //    .map(|f| (f.clone(), pso::VertexInputRate::Vertex))
    //    .chain(Some((VertexArgs::vertex(), pso::VertexInputRate::Instance(1)))).collect::<Vec<_>>();

    let vertex_desc = vec![
        (Position::vertex(), pso::VertexInputRate::Vertex),
        (VertexArgs::vertex(), pso::VertexInputRate::Instance(1)),
    ];

    log::info!("Creating pipeline");

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&vertex_desc)
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: None
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::LessEqual,
                    write: true,
                })
                // This doesn't really work for some reason
                //.with_multisampling(Some(Multisampling {
                //    rasterization_samples: 8,
                //    sample_shading: Some(1.0),
                //    sample_mask: 2,
                //    alpha_coverage: true,
                //    alpha_to_one: false,
                //}))
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