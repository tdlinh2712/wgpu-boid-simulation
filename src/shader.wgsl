// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

@vertex
fn boid_vs_main(
    @location(0) instance_pos: vec2<f32>,
    @location(1) instance_vel: vec2<f32>,
    @location(2) vertex_pos: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let angle = -atan2(instance_vel.x, instance_vel.y);
    let pos = vec2<f32>(
        vertex_pos.x * cos(angle) - vertex_pos.y * sin(angle),
        vertex_pos.x * sin(angle) + vertex_pos.y * cos(angle)
    );
    out.clip_position = vec4<f32>(instance_pos + pos, 0.0, 1.0);
    out.color = vec3<f32>(0.9, 0.6, 0.6); // light pink
}

// Fragment shader

@fragment
fn boid_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
