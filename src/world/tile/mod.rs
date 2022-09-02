pub mod material_identifier;

use df_rust::clients::remote_fortress_reader::remote_fortress_reader::{
    MatPair, Tiletype, TiletypeMaterial, TiletypeShape, TiletypeSpecial, TiletypeVariant,
};

#[derive(Debug)]
pub struct FixedTiletype {
    pub id: i32,
    pub name: Option<String>,
    pub material: TiletypeMaterial,
    pub shape: TiletypeShape,
    pub special: TiletypeSpecial,
    pub variant: TiletypeVariant,
    pub direction: Option<String>,
}

impl From<Tiletype> for FixedTiletype {
    fn from(x: Tiletype) -> Self {
        let material = x.material();
        let shape = x.shape();
        let special = x.special();
        let variant = x.variant();
        Self {
            id: x.id,
            name: x.name.map(|x| String::from_utf8(x).unwrap()),
            direction: x.direction.map(|x| String::from_utf8(x).unwrap()),
            material,
            shape,
            special,
            variant,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Matpair {
    type_: i32,
    index: i32,
}

impl From<MatPair> for Matpair {
    fn from(x: MatPair) -> Self {
        Self {
            index: x.mat_index,
            type_: x.mat_type,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub tile_id: i32,
    pub mat_pair: Matpair,
    pub base_mat: Matpair,
    pub hidden: bool,
}
