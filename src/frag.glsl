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

void solve(float x1, float x2, float x3, out float t1, out float t2, out float t3, out float t4) {
    float a = x1 - 2.0 * x2 + x3;
    float b = x2 - x1;
    float b2 = b * b;
    float q1 = -b + sqrt(max(b2 - a * x1, 0.0));
    float q2 = -b + sqrt(max(b2 - a * (x1 - 1.0), 0.0));
    t1 = clamp(q1 / a, 0.0, 1.0);
    t2 = clamp(x1 / q1, 0.0, 1.0);
    t3 = clamp(q2 / a, 0.0, 1.0);
    t4 = clamp((x1 - 1.0) / q2, 0.0, 1.0);
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
                alpha += (start.x + end.x) * (end.y - start.y);
                continue;
            }

            float t1, t2, t3, t4, t5, t6, t7, t8;
            solve(p1.x, p2.x, p3.x, t1, t2, t3, t4);
            solve(p1.y, p2.y, p3.y, t5, t6, t7, t8);

            int count = 0;
            float points[8];

            if (t1 > 0.0 && t1 < 1.0) {
                points[count] = t1;
                count += 1;
            }
            if (t2 > 0.0 && t2 < 1.0) {
                points[count] = t2;
                count += 1;
            }
            if (t3 > 0.0 && t3 < 1.0) {
                points[count] = t3;
                count += 1;
            }
            if (t4 > 0.0 && t4 < 1.0) {
                points[count] = t4;
                count += 1;
            }
            if (t5 > 0.0 && t5 < 1.0) {
                points[count] = t5;
                count += 1;
            }
            if (t6 > 0.0 && t6 < 1.0) {
                points[count] = t6;
                count += 1;
            }
            if (t7 > 0.0 && t7 < 1.0) {
                points[count] = t7;
                count += 1;
            }
            if (t8 > 0.0 && t8 < 1.0) {
                points[count] = t8;
                count += 1;
            }

            for (int i = 0; i < count; i++) {
                for (int j = i + 1; j < count; j++) {
                    if (points[j] < points[i]) {
                        float tmp = points[i];
                        points[i] = points[j];
                        points[j] = tmp;
                    }
                }
            }

            vec2 prev = start;
            for (int i = 0; i < count; i++) {
                vec2 next = clamp(eval(p1, p2, p3, points[i]), 0.0, 1.0);
                alpha += (prev.x + next.x) * (next.y - prev.y);
                prev = next;
            }
            alpha += (prev.x + end.x) * (end.y - prev.y);
        }
    }

    alpha = clamp(abs(0.5 * alpha), 0.0, 1.0);
    alpha = 1.0 - (1.0 - alpha) * (1.0 - alpha);
    oColor = alpha * vec4(0.f, 0.f, 0.f, 1.f);
}
