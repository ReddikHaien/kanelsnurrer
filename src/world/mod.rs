use std::collections::{btree_map::Entry, BTreeMap};

use bevy::prelude::{Component, Entity};

use self::tile::Tile;

pub mod events;
pub mod tile;
pub mod meshing;

pub struct World {
    chunks: BTreeMap<(i32, i32, i32), Box<Chunk>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: BTreeMap::new(),
        }
    }

    pub fn chunk_mut(&mut self, key: (i32, i32, i32)) -> &mut Chunk {
        match self.chunks.entry(key) {
            Entry::Vacant(entry) => entry.insert(Box::new(Chunk::new())),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
}

pub struct Chunk {
    tiles: [Tile; 4096],
    pub id: Entity,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::default(); 4096],
            id: Entity::from_raw(0),
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, z: i32, tile: Tile) {
        self.tiles[(x + y * 16 + z * 256) as usize] = tile;
    }
}

#[derive(Default, Component)]
pub struct ChunkComponent;
