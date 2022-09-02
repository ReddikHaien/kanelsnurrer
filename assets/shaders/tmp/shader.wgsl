#import bevy_pbr::mesh_functions

struct Vertex{
    @location(0) position<vec4<f32>>
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
}

struct FragmentInput{
    @location(0) color: vec4<f32>
}


@vertex
fn vertex(
    vertex: Vertex
) -> VertexOutput{
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position.xyz, 1.0));
    out.color = vec4<f32>(vertex.w,vertex.w,vertex.w,1.0);

    return out;
}

@fragment
fn fragment(
    input: FragmentInput
) -> @location(0) vec4<f32>{
    return input.color;
}