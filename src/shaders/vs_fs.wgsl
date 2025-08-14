struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct PushConstants {
    mvp_matrix: mat4x4<f32>,
}

var<push_constant> push_constants: PushConstants;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct FragmentIn {
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(vertex_in: VertexIn) -> VertexOut {
    var out_position = push_constants.mvp_matrix * vec4(vertex_in.position, 0.0, 1.0);

    var out: VertexOut;
    out.position = out_position;
    out.uv = vertex_in.uv;
    out.color = vertex_in.color;
    return out;
}

@fragment
fn fs_main(fragment_in: FragmentIn) -> @location(0) vec4<f32> {
    return vec4(fragment_in.color, 1.0);
}
