use bevy::{
    app::Plugin,
    asset::Handle,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::{Commands, Component, Entity, Image, Query, QueryState, ResMut, Resource, World},
    render::{
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{ImageCopyTexture, Origin3d, Texture, TextureAspect},
        texture::GpuImage,
        Extract, ExtractSchedule, RenderApp,
    },
};

pub struct DisplayComputeResultPlugin;

impl Plugin for DisplayComputeResultPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        println!("DisplayComputeResultPlugin");

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, extract_gpu_image_copier);
            render_app.insert_resource(TextureCopiers::default());

            let render_world = render_app.world_mut();

            let node = CopyTextureFromComputeNode::new(render_world);

            if let Some(mut render_graph) = render_world.get_resource_mut::<RenderGraph>() {
                let render_graph = render_graph.sub_graph_mut(Core2d);
                render_graph.add_node(CopyTextureFromComputeLabel, node);
                render_graph.add_node_edge(CopyTextureFromComputeLabel, Node2d::StartMainPass);
            }
        }
    }
}

#[derive(Resource, Default)]
struct TextureCopiers(Vec<Entity>);

#[derive(Component, Clone)]
pub struct TextureCopier {
    pub source: Texture,
    pub target: Handle<Image>,
}

fn extract_gpu_image_copier(
    mut commands: Commands,
    mut copiers: ResMut<TextureCopiers>,
    query: Extract<Query<(Entity, &TextureCopier)>>,
) {
    copiers.0.clear();
    for (entity, copier) in query.iter() {
        let mut commands = commands.get_or_spawn(entity);

        commands.insert(copier.clone());
        copiers.0.push(entity);
    }
}

#[derive(RenderLabel, Hash, Debug, PartialEq, Eq, Clone)]
struct CopyTextureFromComputeLabel;

struct CopyTextureFromComputeNode {
    copiers: QueryState<&'static TextureCopier>,
}

impl CopyTextureFromComputeNode {
    fn new(world: &mut World) -> Self {
        Self {
            copiers: world.query(),
        }
    }
}

impl render_graph::Node for CopyTextureFromComputeNode {
    fn update(&mut self, world: &mut bevy::prelude::World) {
        self.copiers.update_archetypes(world);
    }
    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w bevy::prelude::World,
    ) -> Result<(), render_graph::NodeRunError> {
        let encoder = render_context.command_encoder();
        let copiers = world.resource::<TextureCopiers>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();

        for entity in copiers.0.iter() {
            let Ok(copier) = self.copiers.get_manual(world, *entity) else {
                continue;
            };

            let source = copier.source.clone();
            let target = gpu_images.get(&copier.target).unwrap().texture.clone();

            if (source.size() != target.size())
                | (source.mip_level_count() < 1)
                | (target.mip_level_count() < 1)
            {
                panic!();
            }

            let mut copy_size = source.size();
            copy_size.width = 300;
            copy_size.height = 200;

            encoder.copy_texture_to_texture(
                ImageCopyTexture {
                    texture: &source,
                    mip_level: 0,
                    origin: Origin3d::default(),
                    aspect: TextureAspect::default(),
                },
                ImageCopyTexture {
                    texture: &target,
                    mip_level: 0,
                    origin: Origin3d::default(),
                    aspect: TextureAspect::default(),
                },
                copy_size,
            )
        }

        Ok(())
    }
}
