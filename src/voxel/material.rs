use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::{HandleUntyped, Mat4, Material, Mesh, Shader},
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

pub const VOXEL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x3ec1e761acb8c944);

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct VoxelMaterial {}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_SHADER_HANDLE.typed())
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_SHADER_HANDLE.typed())
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
