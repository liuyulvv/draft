struct VertexInput{
    @location(0) position: vec3f,
    @location(1) tex_coords: vec3f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec3f,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4f(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(
    in:VertexOutput
) -> @location(0) vec4f {
    return vec4f(in.tex_coords, 1.0);
}