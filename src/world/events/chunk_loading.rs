use std::{os::windows::thread, thread::sleep, time::Duration, sync::{RwLock, Mutex}};

use bevy::{prelude::{EventReader, EventWriter, Res, ResMut, Component, Entity, Query, Commands}, tasks::{Task, AsyncComputeTaskPool}, render::once_cell::sync::Lazy};
use df_rust::clients::remote_fortress_reader::{
    remote_fortress_reader::BlockRequest, RemoteFortressReader,
};

use bevy::prelude::IVec3;

use crate::util::{client_pool::ResourcePool, event_queue::EventQueue};

use super::chunk_builder::ChunkBuildEvent;
pub struct ChunkLoadEvent {
    pub entity: Entity,
    pub map_pos: IVec3,
}


static REMOTE_CLIENT: Lazy<Mutex<RemoteFortressReader>> = Lazy::new(||{
    Mutex::new(
        RemoteFortressReader::new(Some("127.0.0.1:5000"))
    )
});

#[derive(Component)]
pub struct LoadData(pub Task<ChunkBuildEvent>);

pub fn create_loader(
    mut reader: EventReader<ChunkLoadEvent>,
    mut query: Query<Entity>,
    mut commands: Commands,
){
    let tp = AsyncComputeTaskPool::get();

    for event in reader.iter(){
    
        let pos = event.map_pos;
        
        let task = tp.spawn(async move{

            let request = BlockRequest {
                blocks_needed: Some(4096),
                min_x: Some(pos.x),
                max_x: Some(pos.x + 1),
                min_y: Some(pos.y),
                max_y: Some(pos.y + 1),
                min_z: Some(pos.z),
                max_z: Some(pos.z + 16),
            };

            let response = {
                let mut client = REMOTE_CLIENT.lock().unwrap();
                client.get_block_list(request)
            };

            ChunkBuildEvent{
                position: pos,
                block: response.map_blocks,
            }
        });
        commands.entity(event.entity)
        .insert(LoadData(task));
    }

}