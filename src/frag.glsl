#version 460

const uint mask = 0xFFF;
const uint shift = 12;

layout(location = 2) uniform usampler2D uComponents;
layout(location = 3) uniform sampler2D uPoints;

layout(location = 0) in vec2 vUv;
layout(location = 1) flat in uvec2 vComponentsRange;
layout(location = 2) flat in uvec2 vPointsRange;

out vec4 oColor;

vec4 fetch(sampler2D tex, uint index) {
    ivec2 coords = ivec2(index & mask, index >> shift);
    return texelFetch(tex, coords, 0);
}

uvec4 ufetch(usampler2D tex, uint index) {
    ivec2 coords = ivec2(index & mask, index >> shift);
    return texelFetch(tex, coords, 0);
}

void main() {
    oColor = vec4(1.f, 0.f, 1.f, 1.f);
    vec2 ddx = dFdx(vUv);
    vec2 ddy = dFdy(vUv);
    mat2x2 view = inverse(mat2(ddx, ddy));

    float alpha = 0.0;
    for (uint i = vComponentsRange.x; i < vComponentsRange.y; i++) {
        uvec2 component = ufetch(uComponents, i).xy;
        for (uint j = vPointsRange.x + component.x; j + 2 < vPointsRange.x + component.y; j += 2) {
            vec2 p1 = view * (fetch(uPoints, j).xy - vUv);
            vec2 p2 = view * (fetch(uPoints, j + 1).xy - vUv);
            vec2 p3 = view * (fetch(uPoints, j + 2).xy - vUv);

            vec2 yWindow = clamp(vec2(p3.y, p1.y), -0.5, 0.5);
            float yOverlap = yWindow.y - yWindow.x;

            float coverage = yOverlap * float(max(p1.x, p3.x) > -0.5);
            if (yOverlap != 0.0 && max(p1.x, p3.x) > -0.5 && min(p1.x, p3.x) < 0.5) {
                float a = p1.y - 2.0 * p2.y + p3.y;
                float b = p2.y - p1.y;
                float c = p1.y - 0.5 * (yWindow.x + yWindow.y);
                float q = -(b + (b < 0.0 ? -1.0 : 1.0) * sqrt(max(b * b - a * c, 0.0)));
                float ta = q / a;
                float tb = c / q;
                float t = (0.0 <= ta && ta <= 1.0) ? ta : tb;
                float x = mix(mix(p1.x, p2.x, t), mix(p2.x, p3.x, t), t);

                vec2 tangent = mix(p2 - p1, p3 - p2, t);
                float f = (x * abs(tangent.y)) / length(tangent);
                float x_overlap = clamp(0.5 + f, 0.0, 1.0);

                coverage *= x_overlap;
            }
            alpha += coverage;
        }
    }

    alpha = clamp(abs(alpha), 0.0, 1.0);
    alpha = 1.0 - (1.0 - alpha) * (1.0 - alpha);
    oColor = alpha * vec4(0.f, 0.f, 0.f, 1.f);
}
