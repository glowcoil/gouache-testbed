#version 460

layout(location = 0) uniform vec2 screen_size;

layout(location = 0) in vec3 pos;

void main() {
    gl_Position = vec4(pos.xy * (2.f / screen_size) - vec2(1.f, 1.f), 0.f, 1.f);
}
