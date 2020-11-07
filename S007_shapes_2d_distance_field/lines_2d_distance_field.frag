precision highp float;
uniform vec2 mouse;           // mouse
uniform float time;           // time
uniform vec2 resolution;      // resolution
uniform sampler2D backbuffer; // prev scene

const float PI = 3.141592;
const float PI2 = PI * 2.;

float cross(vec2 v1, vec2 v2) {
    return dot(v1, vec2(v2.y, -v2.x));
}

float sumabs(vec2 p) {
    return abs(p.x) + abs(p.y);
}
float maxabs(vec2 p) {
    return max(abs(p.x), abs(p.y));
}
mat2 rot(float a) {
    float c = cos(a), s = sin(a);
    return mat2(c, -s, s, c);
}

float op_line(in float sdf, in float r) {
    return step(abs(r - sdf), 0.005);
}

float sdLineInf(vec2 p, float rad, float r) {
    rad -= PI / 2.;
    vec2 v = vec2(p.x * cos(rad) + p.y * sin(rad));
    return length(v);
}

float sdLineInf(vec2 p, vec2 v) {
    return abs(cross(p, normalize(v)));
}

float sdLine(vec2 p, vec2 v1, vec2 v2) {
    vec2 v = v2 - v1;
    vec2 d = normalize(v); //direction
    p -= v1;
    return 1. - (1. - abs(cross(p, d))) * step(0., length(v) * .5 - abs(dot(p - v * .5, d)));
}

float sdLineImpl(vec2 p, vec2 v1, vec2 v2, int cap) {
    vec2 p1 = p - v1;
    vec2 v = v2 - v1;
    float t = dot(p1, normalize(v));
    vec2 pt = (p - ((t < 0.) ? v1 : v2)) * rot(atan(v.x, v.y));
    return (t < 0. || t > length(v))
               ? ((cap == 1) ? length(pt) : (cap == 2) ? maxabs(pt) : (cap == 3) ? sumabs(pt) : 1e10)
               : abs(cross(p1, normalize(v)));
}

float sdLineBat(vec2 p, vec2 v1, vec2 v2) {
    return sdLineImpl(p, v1, v2, 0);
}

float sdLineRound(vec2 p, vec2 v1, vec2 v2) {
    return sdLineImpl(p, v1, v2, 1);
}

float sdLineOverhang(vec2 p, vec2 v1, vec2 v2) {
    return sdLineImpl(p, v1, v2, 2);
}

float sdLineArrow(vec2 p, vec2 v1, vec2 v2) {
    return sdLineImpl(p, v1, v2, 3);
}

void main(void) {
    vec2 p = (gl_FragCoord.xy * 2.0 - resolution) / min(resolution.x, resolution.y);

    float c = 0.;
    c = sdLineInf(p, PI / 6., 0.1);
    c = sdLineInf(p, vec2(1., 0.5));
    c = sdLine(p, vec2(-0.5), vec2(0.5));
    c = sdLineBat(p, vec2(-0.5), vec2(0.5));
    c = sdLineRound(p, vec2(-0.5), vec2(0.5));
    c = sdLineOverhang(p, vec2(-0.5), vec2(0.5));
    c = sdLineArrow(p, vec2(-0.5), vec2(0.5));

    c = op_line(c, 0.1);

    gl_FragColor = vec4(vec3(c), 1.0);
}
