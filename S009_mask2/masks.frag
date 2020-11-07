precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592
#define TAU (PI * 2.)

const float PI2 = PI * 2.;

float circle(vec2 p, float r) {
    return length(p) - r;
}

float op_onion(in float sdf, in float r) {
    return abs(sdf) - r;
}

float op_fill(in float sdf, in float r) {
    return step(0., r - sdf);
}

float pattern_checkers(in vec2 p, in float n) {
    vec2 q = p * n;
    return mod(floor(q.x) + floor(q.y), 2.0);
}

float mask_radial(in vec2 p, in float rad1, in float rad2) {
    float a1 = mod(rad1, PI) / PI;        // -1 to 1
    float a2 = mod(abs(rad2), TAU) / TAU; //  0 to 1
    float a = fract(atan(p.y, p.x) / TAU + 1. - a1);
    return step(a, a2);
}

float mask_range(in float x, in float min, in float max) {
    return step(min, x) * step(x, max);
}

float mask_rect(in vec2 p, in vec2 p1, in vec2 p2) {
    return mask_range(p.x, p1.x, p2.x) * mask_range(p.y, p1.y, p2.y);
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);
    float c = 0.;

    c = op_onion(circle(uv, 0.5), 0.05);
    c = op_fill(c, 0.);

    // c = pattern_checkers(uv, time * 0.5);

    float minx = sin(time * PI + PI / 2.) / 2. - 0.5;
    float maxx = sin(time * PI) / 2. + 0.5;
    float miny = sin(time * PI) / 2. - 0.5;
    float maxy = sin(time * PI) / 2. + 0.5;

    c *= mask_radial(uv, time * PI, PI * 3. / 2.);
    // c *= mask_range(uv.x, minx, maxx);
    // c *= mask_rect(uv, vec2(minx, miny), vec2(maxx, maxy));

    gl_FragColor = vec4(vec3(c), 1.);
}
