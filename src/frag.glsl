#version 410

const uint mask = 0xFFF;
const uint shift = 12;

uniform usampler2D uComponents;
uniform sampler2D uPoints;

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

vec2 eval(vec2 p1, vec2 p2, vec2 p3, float t) {
    return mix(mix(p1, p2, t), mix(p2, p3, t), t);
}

void main() {
    oColor = vec4(1.f, 0.f, 1.f, 1.f);
    vec2 ddx = dFdx(vUv);
    vec2 ddy = dFdy(vUv);
    mat2x2 view = inverse(mat2(ddx, ddy));
    vec2 offset = vec2(0.5, 0.5) - view * vUv;

    float alpha = 0.0;
    for (uint i = vComponentsRange.x; i < vComponentsRange.y; i++) {
        uvec2 component = ufetch(uComponents, i).xy;
        for (uint j = vPointsRange.x + component.x; j + 2 < vPointsRange.x + component.y; j += 2) {
            vec2 p1 = view * (fetch(uPoints, j).xy) + offset;
            vec2 p2 = view * (fetch(uPoints, j + 1).xy) + offset;
            vec2 p3 = view * (fetch(uPoints, j + 2).xy) + offset;

            vec2 start = clamp(p1, 0.0, 1.0);
            vec2 end = clamp(p3, 0.0, 1.0);

            if ((p1.y < 0.0 && p2.y < 0.0 && p3.y < 0.0) ||
                (p1.y > 1.0 && p2.y > 1.0 && p3.y > 1.0) ||
                (p1.x < 0.0 && p2.x < 0.0 && p3.x < 0.0) ||
                (p1.x > 1.0 && p2.x > 1.0 && p3.x > 1.0)) {
                alpha += 0.5 * (start.x + end.x) * (end.y - start.y);
                continue;
            }

            float t1, t2, t3, t4;
            {
                float a = p1.y - 2.0 * p2.y + p3.y;
                float b = p2.y - p1.y;
                float b2 = b * b;
                float sign = b < 0.0 ? -1.0 : 1.0;
                float q1 = -(b + sign * sqrt(max(b2 - a * p1.y, 0.0)));
                float t0a = clamp(q1 / a, 0.0, 1.0);
                float t0b = clamp(p1.y / q1, 0.0, 1.0);

                float q2 = -(b + sign * sqrt(max(b2 - a * (p1.y - 1.0), 0.0)));
                float t1a = clamp(q2 / a, 0.0, 1.0);
                float t1b = clamp((p1.y - 1.0) / q2, 0.0, 1.0);

                float t0min = min(t0a, t0b);
                float t0max = max(t0a, t0b);
                float t1min = min(t1a, t1b);
                float t1max = max(t1a, t1b);
                t1 = min(t0min, t1min);
                t2 = max(t0min, t1min);
                t3 = min(t0max, t1max);
                t4 = max(t0max, t1max);
            }

            float t5, t6, t7, t8;
            {
                float a = p1.x - 2.0 * p2.x + p3.x;
                float b = p2.x - p1.x;
                float b2 = b * b;
                float sign = b < 0.0 ? -1.0 : 1.0;
                float q1 = -(b + sign * sqrt(max(b2 - a * p1.x, 0.0)));
                float t0a = q1 / a;
                float t0b = p1.x / q1;

                float q2 = -(b + sign * sqrt(max(b2 - a * (p1.x - 1.0), 0.0)));
                float t1a = q2 / a;
                float t1b = (p1.x - 1.0) / q2;

                float t0min = min(t0a, t0b);
                float t0max = max(t0a, t0b);
                float t1min = min(t1a, t1b);
                float t1max = max(t1a, t1b);
                t5 = min(t0min, t1min);
                t6 = max(t0min, t1min);
                t7 = min(t0max, t1max);
                t8 = max(t0max, t1max);
            }

            {
                vec2 point1 = clamp(eval(p1, p2, p3, t1), 0.0, 1.0);
                vec2 point2 = clamp(eval(p1, p2, p3, t2), 0.0, 1.0);
                vec2 point5 = clamp(eval(p1, p2, p3, clamp(t5, t1, t2)), 0.0, 1.0);
                vec2 point6 = clamp(eval(p1, p2, p3, clamp(t6, t1, t2)), 0.0, 1.0);
                vec2 point7 = clamp(eval(p1, p2, p3, clamp(t7, t1, t2)), 0.0, 1.0);
                vec2 point8 = clamp(eval(p1, p2, p3, clamp(t8, t1, t2)), 0.0, 1.0);
                alpha += 0.5 * (point1.x + point5.x) * (point5.y - point1.y);
                alpha += 0.5 * (point5.x + point6.x) * (point6.y - point5.y);
                alpha += 0.5 * (point6.x + point7.x) * (point7.y - point6.y);
                alpha += 0.5 * (point7.x + point8.x) * (point8.y - point7.y);
                alpha += 0.5 * (point8.x + point2.x) * (point2.y - point8.y);
            }

            {
                vec2 point3 = clamp(eval(p1, p2, p3, t3), 0.0, 1.0);
                vec2 point4 = clamp(eval(p1, p2, p3, t4), 0.0, 1.0);
                vec2 point5 = clamp(eval(p1, p2, p3, clamp(t5, t3, t4)), 0.0, 1.0);
                vec2 point6 = clamp(eval(p1, p2, p3, clamp(t6, t3, t4)), 0.0, 1.0);
                vec2 point7 = clamp(eval(p1, p2, p3, clamp(t7, t3, t4)), 0.0, 1.0);
                vec2 point8 = clamp(eval(p1, p2, p3, clamp(t8, t3, t4)), 0.0, 1.0);
                alpha += 0.5 * (point3.x + point5.x) * (point5.y - point3.y);
                alpha += 0.5 * (point5.x + point6.x) * (point6.y - point5.y);
                alpha += 0.5 * (point6.x + point7.x) * (point7.y - point6.y);
                alpha += 0.5 * (point7.x + point8.x) * (point8.y - point7.y);
                alpha += 0.5 * (point8.x + point4.x) * (point4.y - point8.y);
            }
        }
    }

    alpha = clamp(abs(alpha), 0.0, 1.0);
    alpha = 1.0 - (1.0 - alpha) * (1.0 - alpha);
    oColor = alpha * vec4(0.f, 0.f, 0.f, 1.f);
}
