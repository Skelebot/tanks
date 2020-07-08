#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

layout(location = 0) in vec3 pos;
layout(location = 1) in mat4 model;     // instance rate
layout(location = 5) in vec4 tint;   // instance rate

layout(location = 0) out VertexData {
    vec3 position;
    vec4 color;
} vertex;

void main() {
    vec4 vertex_position = model * vec4(pos, 1.0);
    vertex.position = vertex_position.xyz;
    vertex.color = tint;
    gl_Position = proj_view * vertex_position;
}