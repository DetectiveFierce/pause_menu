@group(0) @binding(0)
var t_icon: texture_2d<f32>;
@group(0) @binding(1)
var s_icon: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_icon, s_icon, in.uv);
    
    // Only apply antialiasing if the texture has some alpha (not fully transparent)
    if (tex_color.a > 0.0) {
        // Calculate distance from center (assuming circular icon)
        let center = vec2<f32>(0.5, 0.5);
        let distance = length(in.uv - center);
        
        // Create smooth antialiased edge only where the texture is already visible
        let edge_width = 0.01; // Smaller edge width for subtle smoothing
        let circle_alpha = smoothstep(0.5 + edge_width, 0.5 - edge_width, distance);
        
        // Use the minimum of the texture alpha and the circle alpha to avoid adding borders
        let final_alpha = min(tex_color.a, circle_alpha);
        
        return vec4<f32>(tex_color.rgb, final_alpha);
    }
    
    // Return original texture if it's fully transparent
    return tex_color;
} 