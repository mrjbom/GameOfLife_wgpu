struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct PushConstants {
    mvp_matrix: mat4x4<f32>,
}

var<push_constant> push_constants: PushConstants;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(vertex_in: VertexIn) -> VertexOut {
    var out_position = push_constants.mvp_matrix * vec4(vertex_in.position, 1.0);
    var out_color = vertex_in.color;

    var out: VertexOut;
    out.position = out_position;
    out.color = out_color;
    return out;
}

@fragment
fn fs_main(vertex_out: VertexOut) -> @location(0) vec4<f32> {
    return vec4(vertex_out.color, 1.0);
}
