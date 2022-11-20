#version 460

layout(location = 0) uniform vec2 uScreenSize;
layout(location = 1) uniform mat4 uTransform;

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec2 aUv;
layout(location = 2) in uvec2 aComponentsRange;
layout(location = 3) in uvec2 aPointsRange;

layout(location = 0) out vec2 vUv;
layout(location = 1) flat out uvec2 vComponentsRange;
layout(location = 2) flat out uvec2 vPointsRange;

void main() {
    vUv = aUv;
    vComponentsRange = aComponentsRange;
    vPointsRange = aPointsRange;
    gl_Position = uTransform * vec4(aPos, 1.f);
}
