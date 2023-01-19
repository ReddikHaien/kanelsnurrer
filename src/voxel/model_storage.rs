use std::{fmt::{Display, Debug}, mem::MaybeUninit};

use bevy::prelude::{Assets, ResMut, Shader, Resource};
use df_rust::clients::remote_fortress_reader::remote_fortress_reader::TiletypeShape;

use crate::{
    util::{display_iter::DisplayableExt, result_ext::ResultExt, cache::Cache},
    world::tile::material_identifier::{
        MaterialIdentifier, MaterialIdentifierElement, MaterialIdentifierStorage, Identifier,
    },
};

use super::{ModelEntry};

pub struct RegistryContainers<T: >{
    registries: [T;21]
}

impl<T: Default> Default for RegistryContainers<T>{
    fn default() -> Self {
        Self::new(T::default)
    }
}

impl<T> RegistryContainers<T>{

    pub fn new<F: Fn() -> T>(f: F) -> Self{
        
        let mut maybes: [MaybeUninit<_>;21] = MaybeUninit::uninit_array();

        for i in 0..maybes.len(){
            maybes[i].write(f());
        }

        unsafe{
            Self{
                registries: MaybeUninit::array_assume_init(maybes),
            }
        }
        
    }

    pub fn get(&self, shape: TiletypeShape) -> &T{
        let id: i32 = shape.into();
        self.registries.get((id + 1) as usize).unwrap_or_else(|| panic!("unhandled shape id {}, {:?}, index : {}",id,shape, (id + 1) as usize))
    }

    pub fn get_mut(&mut self, shape: TiletypeShape) -> &mut T{
        let id: i32 = shape.into();
        self.registries.get_mut((id + 1) as usize).unwrap_or_else(|| panic!("unhandled shape id {}, {:?}, index : {}",id,shape, (id + 1) as usize))
    }

}

#[derive(Resource)]
pub struct ModelRegistry{
    container: RegistryContainers<ModelStorage>
}

impl ModelRegistry{
    pub fn new() -> Self{
        Self{
            container: RegistryContainers::new(ModelStorage::new)
        }
    }


    pub fn get_storage_entry(&self, shape: TiletypeShape) -> &ModelStorage{
        self.container.get(shape)
    }

    pub fn get_storage_entry_mut(&mut self, shape: TiletypeShape) -> &mut ModelStorage{
        self.container.get_mut(shape)
    }

    pub fn get_model_and_cache(&mut self, id: &Identifier, shape: TiletypeShape) -> Option<&ModelEntry>{
        match self.container.get_mut(shape).get_model_and_cache(id){
            Ok(model) => model,
            Err(model) => {
                if id.last() != Some("STRUCTURAL"){
                    eprintln!("Missing key in {:?} : {}",shape,id);
                }
                model
            },
        }
    }

    pub fn get_model_id(&self, id: &Identifier, shape: TiletypeShape) -> u32 {
        self.container.get(shape).get_model_id(id).either()
    }

    pub fn get_model(&self, id: &Identifier, shape: TiletypeShape) -> Option<&ModelEntry>{
        let model = self.container.get(shape).get_model(id);

        match model {
            Ok(model) => model,
            Err(model) => {
                if shape != TiletypeShape::NoShape{
                    println!("missing model {:?} {:?}",shape, id);
                }
                model
            },
        }

    }
}
pub struct ModelStorage {
    models: Vec<ModelEntry>,

    identifiers: Cache<Identifier,u32>,
}

impl Debug for ModelStorage{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModelStorage").field(&self.identifiers).finish()
    }
}

impl ModelStorage {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            identifiers: Cache::new_with_default(0),
        }
    }

    pub fn get_model_id(&self, id: &Identifier) -> Result<u32,u32> {
        self.identifiers.get_recursive(id).ok_or(0).cloned()
    }

    pub fn get_model(&self, id: &Identifier) -> Result<Option<&ModelEntry>,Option<&ModelEntry>>{
        self.get_model_id(id).map_either(|index| self.models.get((index-1) as usize))
    }

    pub fn get_model_id_and_cache(&mut self, id: &Identifier) -> Result<Option<u32>,Option<u32>>{
        self.identifiers.get_or_initialize_with_parent(id).map_either(|x| x.cloned())
    }

    pub fn get_model_and_cache(&mut self, id: &Identifier) -> Result<Option<&ModelEntry>,Option<&ModelEntry>>{
        self.get_model_id_and_cache(id).map_either(|x|{
            x.map(|y| self.models.get((y-1) as usize)).flatten()
        })
    }

    pub fn add_model(&mut self, model: ModelEntry, identifier: Identifier) {
        self.models.push(model);
        let id = self.models.len() as u32;

        if identifier.is_empty(){
            self.identifiers.set_default(id);
        }
        else{
            self.identifiers
            .set(identifier, id);
        }
    }

    pub fn print_tree(&self) {
        println!("{:#?}",self.identifiers);
    }

    pub fn generate_shader_assets(&self, mut shaders: ResMut<Assets<Shader>>) {
        let shader = format!(
            "
            #import bevy_pbr::mesh_view_bindings
            #import bevy_pbr::mesh_bindings

            #import bevy_pbr::mesh_functions

            struct Vertex{{
                @location(0) position: vec3<f32>,
                @location(1) uvs: vec3<f32>,
            }}

            struct VertexOutput{{
                @builtin(position) clip_position: vec4<f32>,
                @location(0) local_pos: vec3<f32>,
                @location(1) corner: vec3<f32>,
                @location(2) id: u32,
                @location(3) local_camera: vec3<f32>
            }}

            struct FragmentInput{{
                @location(0) local_pos: vec3<f32>,    
                @location(1) corner: vec3<f32>,
                @location(2) id: u32,
                @location(3) local_camera: vec3<f32>
            }}

            @vertex
            fn vertex(
                vertex: Vertex
            ) -> VertexOutput{{
                var out: VertexOutput;

                let v_pos = vec4<f32>(vertex.position, 1.0);

                out.clip_position = mesh_position_local_to_clip(mesh.model, v_pos);

                var i = bitcast<u32>(vertex.uvs.z);

                var x = (i >> 2u) & 1u;
                var y = (i >> 1u) & 1u;
                var z = i & 1u;
                
                let corner = vec3<f32>( f32(x), f32(y), f32(z) );

                let wp = mesh_position_local_to_world(mesh.model, v_pos);

                out.local_pos = vertex.position;
                out.corner = corner;
                out.id = (i >> 3u);

                let lc = transpose(mesh.inverse_transpose_model) * vec4(view.world_position,1.0);
                out.local_camera = lc.xyz / lc.w;

                return out;
            }}

            fn aabb(p: vec3<f32>, position: vec3<f32>, size: vec3<f32>) -> f32{{

                let size = size / 2.0;

                let p = p - position;

                let q = abs(p) - size;

                let w = max(q, vec3<f32>(0.0,0.0,0.0));

                return length(w) + min(max(q.x, max(q.y, q.z)), 0.0);
            }}

            fn sphere(p: vec3<f32>, position: vec3<f32>, radius: f32) -> f32{{
                return distance(p, position) - radius;
            }}

            @fragment
            fn fragment(
                input: FragmentInput
            ) -> @location(0) vec4<f32>{{

                var d: f32 = 0.0;
                let r = normalize(input.local_pos - input.local_camera);   
                var p = input.corner;

                switch(input.id){{
                    case 0u:{{
                        return vec4<f32>(input.corner,1.0);
                    }}
                    {}
                    default: {{
                        return vec4<f32>(0.6,0.2,0.6, 1.0);
                    }}
                }}
            }}
            ",
            self.models
                .iter()
                .enumerate()
                .map(|(id, x)| {
                    CaseObj {
                        id: id + 1,
                        entry: x,
                    }
                })
                .into_displayable('\n')
        );
        //shaders.set_untracked(VOXEL_SHADER_HANDLE, Shader::from_wgsl(shader));
    }
}

struct CaseObj<'a> {
    id: usize,
    entry: &'a ModelEntry,
}

impl<'a> Display for CaseObj<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "case {}u: {{ 
            for (var i = 0; i < 32; i++){{
                d = {};
                p += r*d;
            }}
            if d < 0.01{{
                return vec4<f32>(p,1.0);
            }}
            else{{
                discard;
            }}
         }}",
            self.id, self.entry
        ))
    }
}

//Shape, Tiletype, Material