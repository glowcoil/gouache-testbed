#version 460

layout(location = 0) uniform vec2 screen_size;
layout(location = 1) uniform mat4 transform;

layout(location = 0) in vec3 pos;

void main() {
    gl_Position = transform * vec4(pos, 1.f);
}
