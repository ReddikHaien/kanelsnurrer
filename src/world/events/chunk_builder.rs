use bevy::{
    prelude::{
        default, shape::Cube, Assets, Commands, Entity, EventReader, Handle, IVec3,
        MaterialMeshBundle, Mesh, Query, Res, ResMut, Transform, With,
    },
    render::mesh::VertexAttributeValues, utils::futures,
};
use df_rust::clients::remote_fortress_reader::remote_fortress_reader::{BlockList, MapBlock};
use futures_lite::future;

use crate::{
    voxel::{material::VoxelMaterial, model_storage::ModelStorage},
    world::{
        tile::{material_identifier::MaterialIdentifier, Tile},
        ChunkComponent, World, meshing::build_mesh,
    },
};

use super::chunk_loading::LoadData;

pub struct ChunkBuildEvent {
    pub position: IVec3,
    pub block: Vec<MapBlock>,
}

// pub fn build_chunk(
//     mut events: EventReader<ChunkBuildEvent>,
//     mut commands: Commands,
//     mut world: ResMut<World>,
//     query: Query<(Entity, &mut Handle<Mesh>), With<ChunkComponent>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     models: Res<ModelStorage>,
//     mut materials: ResMut<Assets<VoxelMaterial>>,
// ) {
//     for x in events.iter() {
//         let pos = x.position;
//         let block = &x.block;

//         let pos = IVec3::new(pos.x, pos.z / 16, pos.y);

//         //Flipping coordinates
//         let chunk = world.chunk_mut((pos.x, pos.y, pos.z));

//         for (y, block) in block.into_iter().enumerate(){
//             for x in 0..16 {
//                 for z in 0..16 {
//                     let id = (x + z * 16) as usize;
    
//                     let tile = Tile {
//                         tile_id: block.tiles[id],
//                         mat_pair: block.materials[id].clone().into(),
//                         base_mat: block.base_materials[id].clone().into(),
//                         hidden: block.hidden[id],
//                     };
    
//                     chunk.set_tile(x, y as i32, z, tile)
//                 }
//             }
//         }

        

//         let e = query.iter().find(|(id, _)| id == &chunk.id);

//         match e {
//             Some((_, mesh)) => {
//                 //TODO
//             }
//             None => {
//                 let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                
//                 build_mesh(&mut mesh);

//                 commands.spawn_bundle(MaterialMeshBundle {
//                     mesh: meshes.add(mesh),
//                     material: materials.add(VoxelMaterial {}),
//                     transform: Transform::from_xyz((pos.x * 16) as f32, (pos.y * 16) as f32, (pos.z * 16) as f32),
//                     ..default()
//                 });

//                 println!("generated chunk at {}",pos);
//             }
//         }
//     }
// }

pub fn handle_loading(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LoadData)>,
    mut mesh_query: Query<(Entity, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<World>,
){
    for (entity, mut data) in &mut query{
        if let Some(event) = future::block_on(future::poll_once(&mut data.0)){

            println!("received data for {}",event.position);

            let pos = event.position;
            let block = &event.block;

            let pos = IVec3::new(pos.x, pos.z / 16, pos.y);

            //Flipping coordinates
            let chunk = world.chunk_mut((pos.x, pos.y, pos.z));

            for (y, block) in block.into_iter().enumerate(){
                for x in 0..16 {
                    for z in 0..16 {
                        let id = (x + z * 16) as usize;
        
                        let tile = Tile {
                            tile_id: block.tiles[id],
                            mat_pair: block.materials[id].clone().into(),
                            base_mat: block.base_materials[id].clone().into(),
                            hidden: block.hidden[id],
                        };
        
                        chunk.set_tile(x, y as i32, z, tile)
                    }
                }
            }            

            let mesh = match mesh_query.get(entity){
                Ok((_, mesh)) => {
                    meshes.get_mut(mesh).unwrap()
                },
                Err(_) => {
                    let handle = meshes.add(Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList));
                    commands.entity(entity).insert(handle.clone());
                    meshes.get_mut(&handle).unwrap()      
                }
            };

            build_mesh(mesh);
            commands.entity(entity).remove::<LoadData>();
            println!("Removed LoadData")
        }
    }
}