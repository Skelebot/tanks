#version 450

layout(location = 0) in VertexData {
    vec3 position;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;

void main() {
    if (vertex.color.a < 0.01) discard;
    out_color = vertex.color;
}