struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) quad_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) quad_index: u32,
    @location(1) uv: vec2<f32>,
}

struct Camera {
    x: f32,
    y: f32,
    aspect: f32,
    vertical_height: f32,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct Quad {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    red: f32,
    green: f32,
    blue: f32,
}

@group(1)
@binding(0)
var<storage, read> quads: array<Quad>;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.quad_index = in.quad_index;
    out.uv = vec2<f32>(
        f32((in.vertex_index >> 0u) & 1u),
        f32((in.vertex_index >> 1u) & 1u),
    );

    var vertex_coord = (out.uv * 2.0 - 1.0) * 0.5;
    vertex_coord.x *= quads[in.quad_index].width;
    vertex_coord.y *= quads[in.quad_index].height;
    vertex_coord.x += quads[in.quad_index].x;
    vertex_coord.y += quads[in.quad_index].y;

    out.clip_position = vec4<f32>((vertex_coord - vec2<f32>(camera.x, camera.y)) * camera.vertical_height / vec2<f32>(camera.aspect, 1.0), 0.0, 1.0);

    return out;
}

@fragment
fn pixel(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(quads[in.quad_index].red, quads[in.quad_index].green, quads[in.quad_index].blue, 1.0);
}
