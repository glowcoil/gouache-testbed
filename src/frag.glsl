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

void swap(inout float a, inout float b) {
    float tmp = a;
    a = min(tmp, b);
    b = max(tmp, b);
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
                t1 = clamp(q1 / a, 0.0, 1.0);
                t2 = clamp(p1.y / q1, 0.0, 1.0);

                float q2 = -(b + sign * sqrt(max(b2 - a * (p1.y - 1.0), 0.0)));
                t3 = clamp(q2 / a, 0.0, 1.0);
                t4 = clamp((p1.y - 1.0) / q2, 0.0, 1.0);
            }

            float t5, t6, t7, t8;
            {
                float a = p1.x - 2.0 * p2.x + p3.x;
                float b = p2.x - p1.x;
                float b2 = b * b;
                float sign = b < 0.0 ? -1.0 : 1.0;
                float q1 = -(b + sign * sqrt(max(b2 - a * p1.x, 0.0)));
                t5 = clamp(q1 / a, 0.0, 1.0);
                t6 = clamp(p1.x / q1, 0.0, 1.0);

                float q2 = -(b + sign * sqrt(max(b2 - a * (p1.x - 1.0), 0.0)));
                t7 = clamp(q2 / a, 0.0, 1.0);
                t8 = clamp((p1.x - 1.0) / q2, 0.0, 1.0);
            }

            swap(t1, t2);
            swap(t3, t4);
            swap(t5, t6);
            swap(t7, t8);
            swap(t1, t3);
            swap(t2, t4);
            swap(t5, t7);
            swap(t6, t8);
            swap(t2, t3);
            swap(t6, t7);
            swap(t1, t5);
            swap(t2, t6);
            swap(t3, t7);
            swap(t4, t8);
            swap(t3, t5);
            swap(t4, t6);
            swap(t2, t3);
            swap(t4, t5);
            swap(t6, t7);

            vec2 prev = start;
            vec2 next;
            next = clamp(eval(p1, p2, p3, t1), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t2), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t3), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t4), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t5), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t6), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t7), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; next = clamp(eval(p1, p2, p3, t8), 0.0, 1.0);
            alpha += 0.5 * (prev.x + next.x) * (next.y - prev.y);
            prev = next; alpha += 0.5 * (prev.x + end.x) * (end.y - prev.y);
        }
    }

    alpha = clamp(abs(alpha), 0.0, 1.0);
    alpha = 1.0 - (1.0 - alpha) * (1.0 - alpha);
    oColor = alpha * vec4(0.f, 0.f, 0.f, 1.f);
}
