precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

float sumabs(vec2 p) { return abs(p.x) + abs(p.y); }
float maxabs(vec2 p) { return max(abs(p.x), abs(p.y)); }
float cross(vec2 v1, vec2 v2) { return dot(v1, vec2(v2.y, -v2.x)); }

mat2 opRotate(in float rad) {
    float c = cos(rad);
    float s = sin(rad);
    return mat2(c, -s, s, c);
}

float sdLineImpl(in vec2 p, in vec2 v1, in vec2 v2, in int cap) {
    vec2 p1 = p - v1;
    vec2 v = v2 - v1;
    float t = dot(p1, normalize(v));
    vec2 pt = (p - ((t < 0.) ? v1 : v2)) * opRotate(atan(v.x, v.y));
    return (t < 0. || t > length(v))
               ? ((cap == 1) ? length(pt) : (cap == 2) ? maxabs(pt) : (cap == 3) ? sumabs(pt) : 1e10)
               : abs(cross(p1, normalize(v)));
}

float sdLineBat(in vec2 p, in vec2 v1, in vec2 v2) {
    return sdLineImpl(p, v1, v2, 0);
}

float sdLineRound(in vec2 p, in vec2 v1, in vec2 v2) {
    return sdLineImpl(p, v1, v2, 1);
}

float sdLineArrow(in vec2 p, in vec2 v1, in vec2 v2) {
    return sdLineImpl(p, v1, v2, 3);
}

float op_fill(in float sdf, in float r) {
    return step(0., r - sdf);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution) / min(resolution.x, resolution.y);
    float tiles = 10.;
    vec2 uv_tile = fract(uv * tiles);
    vec2 uv_tile_idx = floor(uv * tiles);
    float rand_tile = floor(time / 4.);

    // change direction
    float dir = rand(uv_tile_idx + rand_tile) > 0.5 ? 1. : 0.;
    vec2 p1 = dir == 1. ? vec2(0.2, 0.2) : vec2(0.2, 0.8);
    vec2 p2 = dir == 1. ? vec2(0.8, 0.8) : vec2(0.8, 0.2);

    float c = 0.;
    c = sdLineBat(uv_tile, p1, p2);
    c = sdLineRound(uv_tile, p1, p2);
    c = sdLineArrow(uv_tile, p1, p2);
    c = op_fill(c, 0.2);

    gl_FragColor = vec4(vec3(c), 1.);
}
