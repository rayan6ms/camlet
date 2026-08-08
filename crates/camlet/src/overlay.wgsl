struct Uniforms {
    surface_size: vec2<f32>,
    frame_offset: vec2<f32>,
    frame_size: vec2<f32>,
    ring_corner: vec2<f32>,
    shape_bounds: vec4<f32>,
    shape: u32,
    _padding_0: u32,
    _padding_1: u32,
    _padding_2: u32,
    ring_color: vec4<f32>,
    accent_color: vec4<f32>,
};

@group(0) @binding(0) var camera: texture_2d<f32>;
@group(0) @binding(1) var camera_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = positions[vertex_index] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return output;
}

fn rounded_box_distance(point: vec2<f32>, half_size: vec2<f32>, requested_radius: f32) -> f32 {
    let radius = clamp(requested_radius, 0.0, min(half_size.x, half_size.y));
    let query = abs(point) - half_size + vec2<f32>(radius);
    return length(max(query, vec2<f32>(0.0))) + min(max(query.x, query.y), 0.0) - radius;
}

fn shape_distance(point: vec2<f32>) -> f32 {
    let half_size = uniforms.shape_bounds.zw * 0.5;
    let local = point - (uniforms.shape_bounds.xy + half_size);
    let corner = uniforms.ring_corner.y;
    if uniforms.shape == 0u {
        return rounded_box_distance(local, half_size, 0.0);
    }
    if uniforms.shape == 1u {
        return length(local) - min(half_size.x, half_size.y);
    }
    if uniforms.shape == 2u {
        return rounded_box_distance(local, half_size, corner);
    }
    if uniforms.shape == 3u {
        let inverse_sqrt_two = 0.7071067811865476;
        let rotated = vec2<f32>(local.x + local.y, -local.x + local.y) * inverse_sqrt_two;
        return rounded_box_distance(rotated, vec2<f32>(min(half_size.x, half_size.y) * inverse_sqrt_two), corner);
    }
    return rounded_box_distance(local, half_size, corner);
}

fn coverage(signed_distance: f32) -> f32 {
    let antialias_width = 1.25;
    return 1.0 - smoothstep(-antialias_width, antialias_width, signed_distance);
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(low, high, color > vec3<f32>(0.04045));
}

fn camera_pixel(point: vec2<f32>) -> vec4<f32> {
    let frame_end = uniforms.frame_offset + uniforms.frame_size;
    if point.x < uniforms.frame_offset.x || point.y < uniforms.frame_offset.y ||
       point.x >= frame_end.x || point.y >= frame_end.y {
        return vec4<f32>(srgb_to_linear(vec3<f32>(6.0, 10.0, 16.0) / 255.0), 1.0);
    }
    var uv = (point - uniforms.frame_offset) / uniforms.frame_size;
    uv.x = 1.0 - uv.x;
    return textureSample(camera, camera_sampler, uv);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let point = input.uv * uniforms.surface_size;
    let distance = shape_distance(point);
    let outer_coverage = coverage(distance);
    if outer_coverage <= 0.0 {
        discard;
    }
    let inner_coverage = min(coverage(distance + uniforms.ring_corner.x), outer_coverage);
    let ring_coverage = max(outer_coverage - inner_coverage, 0.0);
    let frame = camera_pixel(point);
    let camera_alpha = frame.a * inner_coverage;
    let gradient_position = clamp((point.x * 0.68 + point.y * 0.32) / min(uniforms.surface_size.x, uniforms.surface_size.y), 0.0, 1.0);
    let ring = srgb_to_linear(mix(uniforms.ring_color.rgb, uniforms.accent_color.rgb, gradient_position));
    let alpha = clamp(camera_alpha + ring_coverage, 0.0, 1.0);
    let premultiplied = ring * ring_coverage + frame.rgb * camera_alpha;
    return vec4<f32>(premultiplied, alpha);
}
