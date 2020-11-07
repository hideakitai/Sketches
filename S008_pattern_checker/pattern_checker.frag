precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

vec2 uv_even_center(in vec2 uv, in float n) {
    return mod(uv * n, 2.) - 1.;
}

vec2 uv_even_lb(in vec2 uv, in float n) {
    return mod((uv + 1.) * n, 2.) - 1.;
}

vec2 uv_even_rt(in vec2 uv, in float n) {
    return mod((uv - 1.) * n, 2.) - 1.;
}

vec2 uv_even_lt(in vec2 uv, in float n) {
    return mod((uv + vec2(1., -1.)) * n, 2.) - 1.;
}

vec2 uv_even_rb(in vec2 uv, in float n) {
    return mod((uv + vec2(-1., 1.)) * n, 2.) - 1.;
}

vec2 uv_odd_center(in vec2 uv, in float n) {
    return mod(uv * n + 1., 2.) - 1.;
}

float pattern_checkers(in vec2 p, in float n) {
    vec2 q = p * n;
    return mod(floor(q.x) + floor(q.y), 2.0);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // control uv coordinate pattern
    // uv = uv_even_center(uv, time);
    // uv = uv_even_lb(uv, time);
    // uv = uv_even_rt(uv, time);
    // uv = uv_even_lt(uv, time);
    // uv = uv_even_rb(uv, time);
    // uv = uv_odd_center(uv, time);

    float c = 0.;

    // checker pattern
    c = pattern_checkers(uv, time);

    gl_FragColor = vec4(vec3(c), 1.);
}
