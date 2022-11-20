#version 460

layout(location = 2) uniform usampler2D components;
layout(location = 3) uniform sampler2D points;

out vec4 v_col;

void main() {
    v_col = vec4(1.f, 0.f, 1.f, 1.f);
}
