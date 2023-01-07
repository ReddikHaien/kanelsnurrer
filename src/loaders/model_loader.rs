use std::{path::{PathBuf, Path}, fs, error::Error, str::FromStr, hash::Hash};

use bevy::{utils::{HashMap, HashSet, tracing::instrument::WithSubscriber}, prelude::{Vec2, Vec3, Vec4, IVec4, ResMut, AssetServer, Res, Assets, Image, Handle, Resource, App, SystemSet, State, warn}, asset::LoadState, sprite::{TextureAtlasBuilder, TextureAtlas}};
use serde::{Deserialize, Serialize};
use walkdir::{WalkDir, DirEntry};

use crate::{world::tile::material_identifier::MaterialIdentifierElement, voxel::{model_storage::{ModelStorage, ModelRegistry}, ModelEntry, ModelData}};

use super::LoadingInfo;

#[derive(Deserialize)]
pub struct Texturing<'a>{
    src: &'a str,
    clip: Option<(u32,u32,u32,u32)>,
}

#[derive(Deserialize, Serialize,Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Direction{
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backwards,
}

impl Direction{
    pub fn get_coords(self) -> [Vec3;3]{
        match self{
            Direction::Up => [Vec3::X,Vec3::Y,Vec3::Z],
            Direction::Down => [Vec3::X,Vec3::NEG_Y,Vec3::NEG_Z],
            Direction::Left => [Vec3::NEG_Z,Vec3::X,Vec3::NEG_Y],
            Direction::Right => [Vec3::Z,Vec3::NEG_X,Vec3::NEG_Y],
            Direction::Forward => [Vec3::X,Vec3::Z,Vec3::NEG_Y],
            Direction::Backwards => [Vec3::NEG_X,Vec3::NEG_Z,Vec3::NEG_Y],
        }
    }

    pub const fn get_bit(self) -> u8{
        match self {
            Direction::Up =>        0b00100000,
            Direction::Down =>      0b00010000,
            Direction::Left =>      0b00001000,
            Direction::Right =>     0b00000100,
            Direction::Forward =>   0b00000010,
            Direction::Backwards => 0b00000001,
        }
    }

    pub const fn get_bit_offset(self) -> u8{
        match self {
            Direction::Up => 5,
            Direction::Down => 4,
            Direction::Left => 3,
            Direction::Right => 2,
            Direction::Forward => 1,
            Direction::Backwards => 0,
        }
    }
}

#[derive(Deserialize)]
pub enum MeshElement<'a>{
    Params(
        HashMap<&'a str, &'a str>
    ),
    Inherit(&'a str),
    Face{
        n: Direction,
        s: Option<(f32, f32)>,
        p: Option<(f32, f32, f32)>,
        r: Option<f32>,
        t: Texturing<'a>,
        #[serde(default = "Cullable::default")]
        cullable: Cullable
    },
    Mesh{
        verts: Vec<(f32,f32,f32)>,
        uvs: Vec<(f32, f32)>,
        normals: Option<Vec<(f32,f32,f32)>>,
        indices: Vec<(u16,u16,u16)>,
        t: Texturing<'a>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Cullable{
    Never,
    WhenVisible(Direction),
    WhenHidden(Direction),
}

impl Cullable{
    pub fn is_visible(self, mask: u8) -> bool{
        match self{
            Cullable::Never => true,
            Cullable::WhenVisible(direction) => {
                (mask & direction.get_bit()) == 0
            },
            Cullable::WhenHidden(direction) => {
                (mask & direction.get_bit()) == direction.get_bit()
            },
        }
    }
}

impl Default for Cullable {
    fn default() -> Self {
        Self::Never
    }
}

#[derive(Debug)]
pub enum PreBakedModel{
    Face{
        n: Direction,
        s: Vec2,
        p: Vec3,
        r: f32,
        t: (i32, Option<IVec4>),
        cullable: Cullable,
    },
    Mesh{
        verts: Vec<Vec3>,
        uvs: Vec<Vec2>,
        normals: Vec<Vec3>,
        indices: Vec<u16>,
        t: (i32, Option<IVec4>)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BakedModel{
    Quad{
        verts: [Vec3;4],
        uvs: [Vec2;4],
        normal: Vec3,
        cullable: Cullable
    },
}

impl BakedModel{
    pub fn cullable(&self) -> Cullable{
        match self{
            BakedModel::Quad {cullable,..} => *cullable,
        }
    }
}

#[derive(Resource, Default)]
pub struct ModelLoadingData{
    floor_cache: HashMap<PathBuf,Vec<PreBakedModel>>,
    wall_cache: HashMap<PathBuf,Vec<PreBakedModel>>,
    pub atlas_handle: Handle<Image>,
    handlers: Vec<Handle<Image>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
enum ModelLoadingState{
    FileLoading,
    Baking,
}

pub fn add_model_loading(
    app: &mut App
){
    app
    .add_state(ModelLoadingState::FileLoading)
    .insert_resource(ModelLoadingData::default())
    .add_system_set(SystemSet::on_enter(ModelLoadingState::FileLoading)
        .with_system(load_models))
    .add_system_set(SystemSet::on_update(ModelLoadingState::FileLoading)
        .with_system(check_textures))
    .add_system_set(SystemSet::on_enter(ModelLoadingState::Baking)
        .with_system(bake_models));
}

fn check_textures(
    mut  state: ResMut<State<ModelLoadingState>>,
    asset_server: Res<AssetServer>,
    model_data: Res<ModelLoadingData>,
){
    if let LoadState::Loaded = asset_server.get_group_load_state(model_data.handlers.iter().map(|x|x.id())){
        state.set(ModelLoadingState::Baking).unwrap();
    }
}

fn load_models(
    asset_server: Res<AssetServer>,
    mut model_data: ResMut<ModelLoadingData>,
){
    let mut textures = Vec::new();

    model_data.floor_cache = load_folder("assets/materials/floor", &mut textures, &asset_server);
    model_data.wall_cache = load_folder("assets/materials/wall", &mut textures, &asset_server);

    let textures = textures.into_iter().map(|x| asset_server.load::<Image,_>(x)).collect::<Vec<_>>();
    model_data.handlers = textures;
}

fn load_folder(path: &str, textures: &mut Vec<String>, asset_server: &AssetServer)-> bevy::utils::hashbrown::HashMap<PathBuf, Vec<PreBakedModel>> {
    let mut file_cache = HashMap::new();

    let mut materials = Vec::new();

    for file in WalkDir::new(path){
        let Ok(entry) = file else{
            eprintln!("{}",file.unwrap_err());
            continue;
        };

        if entry.file_type().is_file(){
            let path = entry.path();
            materials.push(path.to_owned());
            if !file_cache.contains_key(path){
                load_model(&mut file_cache, entry.into_path())
            }
            else{
                println!("path already loaded! {:?}",path);
            }
        }
    }

    let file_cache = file_cache.into_iter().map(|(path, (vars, mut models))|{
        for x in &mut models{
            match x {
                PreBakedModel::Face { n, s, p, r, t, cullable } => {
                    let texture = solve_variable(&vars, textures, t.0 as usize);
                    t.0 = texture as i32;
                },
                PreBakedModel::Mesh { verts, uvs, normals, indices, t } => {
                    let texture = solve_variable(&vars, textures, t.0 as usize);
                    t.0 = texture as i32;    
                },
            }
        }

        (path, models)
    }).collect::<HashMap<PathBuf,Vec<PreBakedModel>>>();

    println!("{:#?}",textures);
    println!("{:#?}",file_cache);

    
    file_cache
}

pub(super) fn bake_models(
    asset_server: Res<AssetServer>,
    mut model_data: ResMut<ModelLoadingData>,
    mut textures_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
    mut registry: ResMut<ModelRegistry>,
    mut info: ResMut<LoadingInfo>,
){
    let mut atlas_builder = TextureAtlasBuilder::default();
    for x in &model_data.handlers{
        let Some(tex) = textures.get(&x) else{
            warn!("invalid image path {:?}",asset_server.get_handle_path(x));
            continue;
        };

        atlas_builder.add_texture(x.clone_weak(), tex);
    }

    let atlas = atlas_builder.finish(&mut textures).unwrap();
    let atlas_texture = atlas.texture.clone();

    model_data.atlas_handle = atlas_texture;

    registry.floor_storage = build_storage(std::mem::take(&mut model_data.floor_cache), &atlas, &model_data.handlers, "assets/materials/floor");
    registry.wall_storage = build_storage(std::mem::take(&mut model_data.wall_cache), &atlas, &model_data.handlers, "assets/materials/wall");

    info.loaded += 1;
}

fn build_storage(cache: HashMap<PathBuf, Vec<PreBakedModel>>, atlas: &TextureAtlas, handlers: &[Handle<Image>], path_root: &str) -> ModelStorage{

    let mut storage = ModelStorage::new();

    for (path, model) in cache{
        let mut quads = Vec::new();

        for x in model{
            match x{
                PreBakedModel::Face { n, s, p, r, t, cullable } => {
                    let (vs, us, normal,) = create_quad(n, s, p, r);

                    let texture_handle = &handlers[t.0 as usize];
                    let index = atlas.get_texture_index(texture_handle).unwrap();
                    let rect = atlas.textures[index];

                    let off = rect.min / atlas.size;
                    let size = (rect.max - rect.min) / atlas.size;

                    println!("{}: {:?} => {} {}",t.0, texture_handle, off, size);
                    
                    let uv = us.map(|x| x*size + off);
                    quads.push(BakedModel::Quad{
                        verts: vs,
                        uvs: uv,
                        normal,
                        cullable,
                    });
                },
                PreBakedModel::Mesh { verts, uvs, normals, indices, t } => todo!("mesh models not added yet"),
            }
        }

        let name = into_material_name(path, path_root);
        
        let data = ModelData{
            quads,
        };

        storage.add_model(ModelEntry(data), &name);
    }

    storage
}

fn into_material_name(path: PathBuf, root: &str) -> Vec<MaterialIdentifierElement>{

    println!("{}",path.display());
    let path = path.strip_prefix(root).unwrap();

    let mut out = Vec::new();
    for x in path.iter(){
        let s = x.to_string_lossy();
        if s != "mod.ron"{
            let s = if s.ends_with(".ron"){
                s.strip_suffix(".ron").unwrap().to_owned()
            }
            else{
                s.to_string()
            };
            out.push(MaterialIdentifierElement::from(s));
        }
    }

    println!("{:?}",out);
    out

}

fn create_quad(normal: Direction, size: Vec2, position: Vec3, rotation: f32) -> ([Vec3;4], [Vec2;4], Vec3){
    let [x, y, z] = normal.get_coords();

    let x = x/2.0;
    let z = z/2.0;
    let verts = [
        -x*size.x + -z*size.y + position,
         x*size.x + -z*size.y + position,
        -x*size.x +  z*size.y + position,
         x*size.x +  z*size.y + position,
    ];

    let uvs = [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ];
    
    (verts,uvs,y)
}

fn solve_variable(vars: &Vec<(String, String)>, textures: &mut Vec<String>, start: usize) -> usize{
    let mut chain = HashSet::new();
    let mut i = start;
    let texture = loop{
        
        let (key, value) = &vars[i];
        assert!(chain.insert(key),"cyclic reference found {:?}, duplicate key: {}",chain,key);
        if value.starts_with("#"){
            i = vars.iter().enumerate().find(|(_,(k,_))| k == &value[1..]).unwrap().0;
        }
        else{
            break value;
        }
    };
    let texture_index = textures.iter().enumerate().find(|(_,k)|{
        (*k).eq(texture)
    }).map(|(k,_)|k).unwrap_or_else(|| {
        textures.push(texture.clone());
        textures.len()-1
    });

    texture_index
}

fn load_model(file_cache: &mut HashMap<PathBuf,(Vec<(String,String)>,Vec<PreBakedModel>)>, path: PathBuf){
    println!("reading {:?}",path.as_os_str());
    let source = fs::read_to_string(&path).unwrap();
    let raw_model: Vec<MeshElement> = ron::from_str(&source).unwrap();

    let mut prebaked = Vec::new();

    let mut variables = Vec::new();

    for x in raw_model{
        match x{
            MeshElement::Params(vars) => {
                for (key, value) in vars{
                    variables.push((key.to_owned(), value.to_owned()));
                }
            },
            MeshElement::Inherit(src) => {
                let inherit_path = PathBuf::from_str(src).unwrap();
                if !file_cache.contains_key(&inherit_path){
                    load_model(file_cache, inherit_path.clone());
                }
                let parent = file_cache.get(&inherit_path).unwrap();
                'outer: for (key, value) in &parent.0{
                    for (exsisting_key, _) in &variables{
                        if key == exsisting_key{
                            continue 'outer;
                        }
                    }

                    variables.push((key.clone(), value.clone()));
                }

                for x in &parent.1{
                    match x{
                        PreBakedModel::Face { n, s, p, r, t, cullable } => {
                            let var_key = &parent.0[t.0 as usize].0;
                            let mut t = (t.0, t.1);
                            if let Some ((index,_)) = variables.iter().enumerate().find(|(_, (key,_))| key == var_key){
                                t.0 = index as i32;
                            }
                            
                            prebaked.push(PreBakedModel::Face{
                                n: *n,
                                p: *p,
                                r: *r,
                                s: *s,
                                t,
                                cullable: *cullable
                            });
                        },
                        PreBakedModel::Mesh { verts, uvs, normals, indices, t } => {
                            let var_key = &parent.0[t.0 as usize].0;
                            let mut t = (-1, t.1);
                            if let Some ((index,_)) = variables.iter().enumerate().find(|(_, (key,_))| key == var_key){
                                t.0 = index as i32;
                            }

                            prebaked.push(PreBakedModel::Mesh{
                                indices: indices.clone(),
                                normals: normals.clone(),
                                uvs: uvs.clone(),
                                verts: verts.clone(),
                                t
                            });
                        },
                    }
                }
            },
            MeshElement::Face { n, s, p, r, t , cullable} => {
                let t = {
                    let index = variables
                        .iter()
                        .enumerate()
                        .find_map(|x| if x.1.0 == t.src {Some(x.0 as i32)} else {None})
                        .unwrap_or_else(|| panic!("unreqognized variable {}",t.src));

                    (index,t.clip.map(|(x, y, z, w)| IVec4::new(x as i32, y as i32, z as i32, w as i32)))
                };

                let n = n;
                let s = Vec2::from(s.unwrap_or((1.0,1.0)));
                let p = Vec3::from(p.unwrap_or((0.0,0.0,0.0)));
                let r = r.unwrap_or(0.0);
                
                prebaked.push(PreBakedModel::Face{
                    n,
                    p,
                    r,
                    s,
                    t,
                    cullable
                });
            },
            MeshElement::Mesh { verts, uvs, normals, indices, t } => {
                let t = {
                    let index = variables
                        .iter()
                        .enumerate()
                        .find_map(|x| if x.1.0 == t.src {Some(x.0 as i32)} else {None})
                        .unwrap_or_else(|| panic!("unreqognized variable {}",t.src));

                    (index,t.clip.map(|(x, y, z, w)| IVec4::new(x as i32, y as i32, z as i32, w as i32)))
                };

                let verts: Vec<Vec3> = verts.into_iter().map(|x| x.into()).collect();
                let uvs: Vec<Vec2> = uvs.into_iter().map(|x| x.into()).collect();
                let normals: Vec<Vec3> = normals.unwrap().into_iter().map(|x|x.into()).collect();
                let indices: Vec<u16> = indices.into_iter().flat_map(|x| [x.0,x.1,x.2].into_iter()).collect();

                prebaked.push(PreBakedModel::Mesh{
                    indices,
                    normals,
                    t,
                    uvs,
                    verts
                });
            },
        }
    }

    file_cache.insert(path, (variables,prebaked));
}