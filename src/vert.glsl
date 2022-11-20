#version 460

layout(location = 0) uniform vec2 uScreenSize;
layout(location = 1) uniform mat4 uTransform;

layout(location = 0) in vec3 aPos;

void main() {
    gl_Position = uTransform * vec4(aPos, 1.f);
}
