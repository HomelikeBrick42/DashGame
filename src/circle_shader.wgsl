struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) circle_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) circle_index: u32,
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

struct Circle {
    x: f32,
    y: f32,
    radius: f32,
    red: f32,
    green: f32,
    blue: f32,
}

@group(1)
@binding(0)
var<storage, read> circles: array<Circle>;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.circle_index = in.circle_index;
    out.uv = vec2<f32>(
        f32((in.vertex_index >> 0u) & 1u),
        f32((in.vertex_index >> 1u) & 1u),
    );

    var vertex_coord = out.uv * 2.0 - 1.0;
    vertex_coord *= circles[in.circle_index].radius;
    vertex_coord.x += circles[in.circle_index].x;
    vertex_coord.y += circles[in.circle_index].y;

    out.clip_position = vec4<f32>((vertex_coord - vec2<f32>(camera.x, camera.y)) / camera.vertical_height * 2.0 / vec2<f32>(camera.aspect, 1.0), 0.0, 1.0);

    return out;
}

@fragment
fn pixel(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = in.uv * 2.0 - 1.0;
    if dot(coord, coord) > 1.0 {
        discard;
    }
    return vec4<f32>(circles[in.circle_index].red, circles[in.circle_index].green, circles[in.circle_index].blue, 1.0);
}
