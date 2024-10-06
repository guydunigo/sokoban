struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist_x = in.uv[0] - 0.5;
    var dist_y = in.uv[1] - 0.5;
    var dist = (1. - fract(sqrt(dist_x * dist_x + dist_y * dist_y)));
    var rand = (1. - rand(in.uv) / 2.) * dist * dist;
    return textureSample(t, s, in.uv) * vec4<f32>(rand, rand, rand * 0.5, 1.);
}

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}
