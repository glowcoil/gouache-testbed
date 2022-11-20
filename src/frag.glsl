#version 460


layout(location = 2) uniform usampler2D uComponents;
layout(location = 3) uniform sampler2D uPoints;

layout(location = 0) in vec2 vUv;
layout(location = 1) flat in uvec2 vComponentsRange;
layout(location = 2) flat in uvec2 vPointsRange;

out vec4 oColor;

void main() {
    oColor = vec4(1.f, 0.f, 1.f, 1.f);
}
