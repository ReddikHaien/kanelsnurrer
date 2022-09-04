use std::collections::{btree_map::Entry, BTreeMap};

use bevy::prelude::{Component, Entity, FromWorld};
use df_rust::clients::remote_fortress_reader::{RemoteFortressReader, remote_fortress_reader::{MatPair, TiletypeMaterial, TiletypeShape, TiletypeSpecial, TiletypeVariant, Tiletype}};

use self::tile::{Tile, material_identifier::MaterialIdentifier};

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

    pub fn chunk(&self, key: (i32, i32, i32)) -> &Chunk{
        match self.chunks.get(&key) {
            Some(chunk) => &chunk,
            None => panic!("no chunk at {:?}",key),
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

    pub fn tile_ref(&self, x: i32, y: i32, z: i32) -> &Tile{
        &self.tiles[(x + y * 16 + z * 256) as usize]
    }
}

#[derive(Default, Component)]
pub struct ChunkComponent;


#[derive(Debug,Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Matpair{
    pub type_: i32,
    pub index: i32,
}

impl From<MatPair> for Matpair{
    fn from(x: MatPair) -> Self {
        Matpair{
            index: x.mat_index,
            type_: x.mat_type,
        }
    }
}

#[derive(Debug)]
pub struct MaterialDef{
    pub id: Option<MaterialIdentifier>,
    pub mat_pair: Matpair,
}

#[derive(Debug)]
pub struct FixedTiletype{
    pub id: i32,
    pub name: Option<String>,
    pub material: TiletypeMaterial,
    pub shape: TiletypeShape,
    pub special: TiletypeSpecial,
    pub variant: TiletypeVariant,
    pub direction: Option<String>
}

impl From<Tiletype> for FixedTiletype{
    fn from(x: Tiletype) -> Self {
        let material = x.material();
        let shape = x.shape();
        let special = x.special();
        let variant = x.variant();
        Self{
            id: x.id,
            name: x.name.map(|x| String::from_utf8(x).unwrap()),
            direction: x.direction.map(|x| String::from_utf8(x).unwrap()),
            material,
            shape,
            special,
            variant
        }
    }
}

pub struct MaterialRegistry{
    matdefs: BTreeMap<Matpair, MaterialDef>,
    tiletypes: Vec<FixedTiletype>
}

impl MaterialRegistry{

    pub fn get_tiletype(&self, tile: &Tile) -> &FixedTiletype{
        &self.tiletypes[tile.tile_id as usize]
    }
}

impl FromWorld for MaterialRegistry{
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut client = world.get_resource_mut::<RemoteFortressReader>().unwrap();
        let matdefs: BTreeMap<Matpair, MaterialDef> = client.get_material_list().material_list.into_iter().map(
            |x|{
                let id = x.id.map(
                    |y|
                    MaterialIdentifier::from(String::from_utf8(y).unwrap())
                );
                let mp = x.mat_pair.into();
                (
                    mp,
                    MaterialDef{
                        id,
                        mat_pair: mp
                    }
                )
            }
        ).collect();

        let tiletypes = client.get_tile_type_list().tiletype_list.into_iter().map(|x| x.into()).collect();

        Self {
            matdefs,
            tiletypes
        }
    }
}