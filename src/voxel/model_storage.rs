use std::fmt::{Display, Debug};

use bevy::prelude::{Assets, ResMut, Shader, Resource};
use df_rust::clients::remote_fortress_reader::remote_fortress_reader::TiletypeShape;

use crate::{
    util::{display_iter::DisplayableExt, result_ext::ResultExt},
    world::tile::material_identifier::{
        MaterialIdentifier, MaterialIdentifierElement, MaterialIdentifierStorage,
    },
};

use super::{material::VOXEL_SHADER_HANDLE, ModelEntry};

#[derive(Resource)]
pub struct ModelRegistry{
    pub floor_storage: ModelStorage,
    pub wall_storage: ModelStorage,
    pub up_down_stair_storage: ModelStorage,
    pub down_stair_storage: ModelStorage,
}

impl ModelRegistry{
    pub fn new() -> Self{
        Self{
            floor_storage: ModelStorage::new(),
            wall_storage: ModelStorage::new(),
            down_stair_storage: ModelStorage::new(),
            up_down_stair_storage: ModelStorage::new(),
        }
    }

    pub fn get_model_id(&self, id: &MaterialIdentifier, shape: TiletypeShape) -> u32 {
        let model = match shape {
            TiletypeShape::Floor => self.floor_storage.get_model_id(id),
            TiletypeShape::Wall => self.wall_storage.get_model_id(id),
            TiletypeShape::StairDown => self.down_stair_storage.get_model_id(id),
            TiletypeShape::StairUpdown => self.up_down_stair_storage.get_model_id(id),
            _ => Ok(0)
        };
        
        model.either()
    }

    pub fn get_model(&self, id: &MaterialIdentifier, shape: TiletypeShape) -> Option<&ModelEntry>{
        let model = match shape {
            TiletypeShape::Floor => self.floor_storage.get_model(id),
            TiletypeShape::Wall => self.wall_storage.get_model(id),
            TiletypeShape::StairDown => self.down_stair_storage.get_model(id),
            TiletypeShape::StairUpdown => self.up_down_stair_storage.get_model(id),
            _ => Ok(None)
        };

        match model {
            Ok(model) => model,
            Err(model) => {
                println!("missing model {:?} {:?}",shape, id);
                model
            },
        }

    }
}
pub struct ModelStorage {
    models: Vec<ModelEntry>,

    identifiers: MaterialIdentifierStorage,
}

impl ModelStorage {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            identifiers: MaterialIdentifierStorage::new(),
        }
    }

    pub fn get_model_id(&self, id: &MaterialIdentifier) -> Result<u32,u32> {
        self.identifiers.get_id(id)
    }

    pub fn get_model(&self, id: &MaterialIdentifier) -> Result<Option<&ModelEntry>,Option<&ModelEntry>>{
        self.get_model_id(id).map_either(|index| self.models.get((index-1) as usize))
    }

    pub fn add_model(&mut self, model: ModelEntry, identifier: &Vec<MaterialIdentifierElement>) {
        self.models.push(model);
        let id = self.models.len();
        self.identifiers
            .set_id(&identifier.clone().into(), id as u32);
    }

    pub fn print_tree(&self) {
        self.identifiers.print_tree();
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
        shaders.set_untracked(VOXEL_SHADER_HANDLE, Shader::from_wgsl(shader));
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