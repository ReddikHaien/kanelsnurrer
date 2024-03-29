use bevy::{prelude::{Mesh, Vec3, IVec3, Vec2}, render::mesh::Indices};

use crate::voxel::model_storage::ModelRegistry;

use super::{MaterialRegistry, World, Chunk};

pub fn build_mesh(
    mesh: &mut Mesh,
    chunk: &Chunk,
    registry: &MaterialRegistry,
    models: &mut ModelRegistry){
    let mut verts = Vec::<Vec3>::new();
    let mut uvs = Vec::<Vec2>::new();
    let mut normals = Vec::<Vec3>::new();
    let mut indices = Vec::<u16>::new();

    for x in 0..16{
        for y in 0..16{
            for z in 0..16{
                let tile = chunk.tile_ref(x, y, z);
                
                if !tile.hidden{
                    let pos = IVec3::new(x,y,z).as_vec3();
                    let type_ = registry.get_tiletype(tile);

                    let mat_pair = &tile.base_mat;
                    let Some(def) = registry.matdefs.get(mat_pair)
                        else{
                            continue;
                        };
                    let id = def.id.as_ref().unwrap();

                    let Some(model) = models.get_model_and_cache(id, type_.shape) else{ continue; };
                    
                    let mask = chunk.get_mask(x, y, z, &registry);

                    model.0.models.iter().filter(|x| x.cullable().is_visible(mask)).for_each(|x|{
                        match x {
                            crate::loaders::model_loader::BakedModel::Quad { verts:v, uvs:u, normal:n, cullable:_ } => {
                                let c = verts.len() as u16;
                                verts.extend(v.iter().map(|x| *x + pos));
                                uvs.extend(u);
                                normals.extend([*n;4]);
                                indices.extend([
                                    c + 0u16, c + 2, c + 1,
                                    c + 2,    c + 3, c + 1
                                ]);
                            },
                            crate::loaders::model_loader::BakedModel::Mesh { data, indices:i, cullable:_ } => {
                                let c = verts.len() as u16;
                                for (v, u, n) in data{
                                    verts.push(*v + pos);
                                    uvs.push(*u);
                                    normals.push(*n);
                                }
                                indices.extend(i.iter().map(|x| *x + c));
                            },
                            
                        }
                    });
                }
            }
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,  normals);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.compute_aabb();
}