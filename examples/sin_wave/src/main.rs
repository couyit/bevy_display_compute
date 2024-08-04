use bevy::{
    app::{App, Startup},
    asset::{Assets, Handle},
    prelude::{Camera2dBundle, Commands, Image, In, IntoSystem, NodeBundle, Res, ResMut},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
    },
    ui::{AlignItems, IsDefaultUiCamera, JustifyContent, Style, UiImage, Val},
    utils::default,
    DefaultPlugins,
};
use bevy_display_compute::{DisplayComputeResultPlugin, TextureCopier};

pub fn main() {
    println!("SINWAVE");

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, DisplayComputeResultPlugin));
    app.add_systems(Startup, setup.pipe(gui_setup));
    app.run();
}

fn setup(
    mut command: Commands,
    device: Res<RenderDevice>,
    mut images: ResMut<Assets<Image>>,
) -> Handle<Image> {
    command.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

    let size = Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 1,
    };

    let dimension = TextureDimension::D2;

    let format = TextureFormat::Rgba32Float;

    let pixel = bytemuck::cast_slice(&[1.0f32, 0.0f32, 0.5f32, 1.0f32]);

    let image = Image::new_fill(
        size,
        dimension,
        pixel,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );

    let handle = images.add(image);

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension,
        format,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    // let gpu_image = bevy_gpu_image_creation_utils::create_gpu_image_from_texture(&device, texture);

    command.spawn(TextureCopier {
        source: texture,
        target: handle.clone(),
    });

    handle
}

fn gui_setup(In(image): In<Handle<Image>>, mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(512.0),
                height: Val::Px(512.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
        UiImage::new(image),
    ));
}
