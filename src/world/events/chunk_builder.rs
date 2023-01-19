use bevy::{
    prelude::{
        default, Assets, Commands, Entity, Handle, IVec3,
        MaterialMeshBundle, Mesh, Query, Res, ResMut, Transform, StandardMaterial, PbrBundle, HandleUntyped, Material,
    }, reflect::TypeUuid,
};
use df_rust::clients::remote_fortress_reader::remote_fortress_reader::{MapBlock, TiletypeShape};
use futures_lite::future;

use crate::{
    voxel::{model_storage::{ModelStorage, ModelRegistry}},
    world::{
        tile::Tile, World, meshing::build_mesh, MaterialRegistry,
    }, loaders::model_loader::ModelLoadingData,
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

pub const VOXEL_MATERIAL: HandleUntyped = 
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 12012309628019059972);

pub fn handle_loading(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LoadData)>,
    mesh_query: Query<(Entity, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<World>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_registry: Res<MaterialRegistry>,
    mut model_storage: ResMut<ModelRegistry>,
    model_data: Res<ModelLoadingData>
){

    

    for (entity, mut data) in &mut query{
        if let Some(event) = future::block_on(future::poll_once(&mut data.0)){

            //println!("received data for {}",event.position);

            let pos = event.position;
            let block = &event.block;

            let df_z_coord = pos.z;

            let pos = IVec3::new(pos.x, pos.z / 16, pos.y);



            {
                let chunk = world.chunk_mut((pos.x, pos.y, pos.z));
                
                for block in block{
                    let y = block.map_z - df_z_coord;
                    assert!(y >= 0);
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
            }
            
            let mesh = match mesh_query.get(entity){
                Ok((_, mesh)) => {
                    meshes.get_mut(mesh).unwrap()
                },
                Err(_) => {
                    let handle = meshes.add(Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList));
                    commands.entity(entity)
                    .insert(PbrBundle{
                        mesh: handle.clone(),
                        material: VOXEL_MATERIAL.typed().clone(),
                        transform: Transform::from_xyz((pos.x * 16) as f32, (pos.y * 16) as f32, (pos.z * 16) as f32),
                        ..default()
                    });
                    
                    meshes.get_mut(&handle).unwrap()      
                }
            };

            let chunk = world.chunk((pos.x,pos.y,pos.z));

            build_mesh(
                mesh,
                &chunk,
                &material_registry,
                &mut model_storage
            );
            commands.entity(entity).remove::<LoadData>();
            //println!("Removed LoadData for {}",pos);
            //model_storage.get_storage_entry(TiletypeShape::Wall).print_tree();
        }
    }
}