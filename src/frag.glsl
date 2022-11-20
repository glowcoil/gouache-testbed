#version 460


layout(location = 2) uniform usampler2D uComponents;
layout(location = 3) uniform sampler2D uPoints;

out vec4 oColor;

void main() {
    oColor = vec4(1.f, 0.f, 1.f, 1.f);
}
