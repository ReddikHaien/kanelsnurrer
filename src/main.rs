#![feature(iterator_try_collect)]
mod loaders;
pub mod util;
pub mod voxel;
pub mod world;

use bevy::{
    prelude::{
        default, App, Assets, Camera3dBundle, Commands, EventWriter,
        Handle, IVec3, MaterialPlugin, Mesh, Query, Res, ResMut, SystemSet, Transform, Vec3, With, Camera, Input, KeyCode, Resource, ImagePlugin, PluginGroup, DirectionalLightBundle, AmbientLight, Color, DirectionalLight,
    },
    DefaultPlugins, time::Time,
};
use df_rust::clients::remote_fortress_reader::RemoteFortressReader;
use voxel::{material::VoxelMaterial, model_storage::{ModelStorage, ModelRegistry}};
use world::{
    events::{
        chunk_builder::{ChunkBuildEvent, handle_loading},
        chunk_loading::{ChunkLoadEvent, create_loader},
    },
    World, MaterialRegistry,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Setup,
    Running,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(MaterialPlugin::<VoxelMaterial>::default())
        .insert_resource(ModelRegistry::new())
        .insert_resource(AmbientLight{
            brightness: 0.04,
            color: Color::WHITE
        })
        .insert_resource(FortressResource(RemoteFortressReader::new(Some("127.0.0.1:5000"))))
        .add_state(AppState::Setup)
        .insert_resource(World::new())
        .init_resource::<MaterialRegistry>()
        .add_event::<ChunkBuildEvent>()
        .add_event::<ChunkLoadEvent>()
        .add_system_set(SystemSet::on_enter(AppState::Running).with_system(startup_system))
        .add_system_set(
            SystemSet::on_update(AppState::Running)
                //.with_system(rotator)
                
                .with_system(create_loader)
                .with_system(handle_loading)
                .with_system(camera_mover)
        );

    loaders::add_loading_methods(&mut app).run();
}

fn rotator(mut query: Query<(&mut Transform, With<Handle<Mesh>>)>) {
    for (mut transform, _) in query.iter_mut() {
        transform.rotate_axis(Vec3::Y, 0.01);
    }
}

#[derive(Resource)]
pub struct FortressResource(pub RemoteFortressReader);

fn startup_system(
    mut client: ResMut<FortressResource>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut writer: EventWriter<ChunkLoadEvent>,
    models: Res<ModelRegistry>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-10.0, 185.0, -10.0).looking_at(Vec3::new(0.0, 180.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle{
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
        directional_light: DirectionalLight{
            color: Color::rgb(1.0, 1.0, 0.8),
            ..default()
        },
        ..default()
    });

    client.0.reset_map_hashes();

    let info = client.0.get_map_info();

    for x in 0..info.block_size_x() {
        for y in 0..info.block_size_y() {
            for z in (0..info.block_size_z()).step_by(16) {
                writer.send(ChunkLoadEvent {
                    entity: commands.spawn_empty().id(),
                    map_pos: IVec3::new(x, y, z),
                });
            }
        }
    }
}


fn camera_mover(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera>>
){
    for mut transform in query.iter_mut(){
        
        if input.pressed(KeyCode::Left){
            transform.rotate_y(2.0 * time.delta_seconds());
        }

        if input.pressed(KeyCode::Right){
            transform.rotate_y(-2.0 * time.delta_seconds());
        }

        if input.pressed(KeyCode::Up){
            transform.rotate_local_x(2.0 * time.delta_seconds());
        }

        if input.pressed(KeyCode::Down){
            transform.rotate_local_x(-2.0 * time.delta_seconds());
        }

        let fw = transform.forward();
        let r = transform.right();
        if input.pressed(KeyCode::W){
            transform.translation += fw * 2.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::S){
            transform.translation += fw * -2.0 * time.delta_seconds();
        }

        if input.pressed(KeyCode::A){
            transform.translation += r * -2.0 * time.delta_seconds();
        }
        if input.pressed(KeyCode::D){
            transform.translation += r * 2.0 * time.delta_seconds();
        }
    }
}