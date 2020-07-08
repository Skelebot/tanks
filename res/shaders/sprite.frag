#version 450

in vec4 gl_FragCoord;

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec2 uv;
    vec4 tint;
    vec4 tintbox;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {
    float end_x = vertex.tintbox.x + vertex.tintbox.z;
    float end_y = vertex.tintbox.y + vertex.tintbox.a;

    vec4 color = texture(albedo, vertex.tex_uv) * vec4(1.0, 1.0, 1.0, vertex.tint.a);

    if (vertex.uv.x >= vertex.tintbox.x && vertex.uv.y >= vertex.tintbox.y && vertex.uv.x <= end_x && vertex.uv.y <= end_y) {
        color *= vec4(vertex.tint.xyz, 1.0); 
    }

    //vec4 color = vec4(vertex.uv.x, vertex.uv.y, vertex.tintbox.x, 1.0);

    if (color.a == 0.0) {
        discard;
    }
    out_color = color;
}