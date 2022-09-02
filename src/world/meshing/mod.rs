use bevy::{prelude::{Mesh, Vec3, IVec3}, render::mesh::Indices};

pub fn build_mesh(mesh: &mut Mesh){
    let mut verts = Vec::new();
    let mut data = Vec::new();
    let mut indices = Vec::new();
    let mut fcount = 0;
    for x in 0..16{
        for z in 0..16{
            verts.push(IVec3::new(x, 1, z).as_vec3().to_array());
            verts.push(IVec3::new(x+1, 1, z).as_vec3().to_array());
            verts.push(IVec3::new(x, 1, z+1).as_vec3().to_array());
            verts.push(IVec3::new(x+1, 1, z+1).as_vec3().to_array());
        

            data.push([f32::from_bits(0),f32::from_bits(0),
                f32::from_bits(
                    0 << 3 | 0 << 2 | 1 << 1 | 0,
                )]);
            data.push([f32::from_bits(0),f32::from_bits(0),
                f32::from_bits(
                    0 << 3 | 1 << 2 | 1 << 1 | 0,
                )]);
            data.push([f32::from_bits(0),f32::from_bits(0),
                f32::from_bits(
                    0 << 3 | 0 << 2 | 1 << 1 | 1,
                )]);
            data.push([f32::from_bits(0),f32::from_bits(0),
                f32::from_bits(
                    0 << 3 | 1 << 2 | 1 << 1 | 1,
                )]);

            indices.push(fcount);
            indices.push(fcount+2);
            indices.push(fcount+1);
            indices.push(fcount+2);
            indices.push(fcount+3);
            indices.push(fcount+1);

            fcount += 4;
        }
    }
    mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
    mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.compute_aabb();
}