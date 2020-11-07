precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

const float PI = acos(-1.);
const float TAU = PI * 2.;

float circle(vec2 p, float r) {
    return length(p) - r;
}

float square(vec2 p, float r) {
    return max(abs(p.x), abs(p.y)) - r;
}

float rect(vec2 p, vec2 size) {
    vec2 d = abs(p) - size;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

float polygon(vec2 p, float n) {
    float a = atan(p.x, p.y) + PI;
    float r = TAU / n;
    return cos(floor(.5 + a / r) * r - a) * length(p) / cos(r * .5);
}

float lengthN(vec2 p, float n) {
    vec2 tmp = pow(abs(p), vec2(n));
    return pow(tmp.x + tmp.y, 1.0 / n);
}

float asteroid(vec2 p, float r) {
    return lengthN(p, 0.5) - r;
}

float op_fill(in float sdf, in float r) {
    return step(0., r - sdf);
}

float op_line(in float sdf, in float r) {
    return step(abs(r - sdf), 0.005);
}

float op_grow_fill(in float sdf, in float r) {
    return r / sdf;
}

float op_grow(in float sdf, in float r) {
    return 0.01 / abs(sdf - r);
}

float op_grow_outer(in float sdf, in float r) {
    return 0.01 / (sdf - r);
}

float op_grow_inner(in float sdf, in float r) {
    return 0.01 / -(sdf - r);
}

float op_fill_antialias(in float sdf, in float r) {
    return smoothstep(0., 0.01, r - sdf);
}

float op_line_antialias(in float sdf, in float r) {
    return 1. - smoothstep(0.005, 0.015, abs(r - sdf));
}

float op_fill_gradation(in float sdf, in float r) {
    return smoothstep(0., 0.5, r - sdf);
}

float op_line_gradation(in float sdf, in float r) {
    return 1. - smoothstep(0.005, 0.5, abs(r - sdf));
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution) / min(resolution.x, resolution.y);

    float c = 0.;
    c = circle(uv, 0.3);
    c = square(uv, 0.5);
    c = rect(uv, vec2(0.4, 0.2));
    c = polygon(uv, 3.) - 0.5;

    // c = sdEquilateralTriangle(uv * 2.);

    // float op_fill(in float sdf, in float r);
    // float op_line(in float sdf, in float r);
    // float op_grow_fill(in float sdf, in float r);
    // float op_grow(in float sdf, in float r);
    // float op_grow_outer(in float sdf, in float r);
    // float op_grow_inner(in float sdf, in float r);
    // float op_line_antialias(in float sdf, in float r);
    // float op_line_antialias2(in float sdf, in float r);
    // float op_gradation(in float sdf, in float r);
    // float op_gradation2(in float sdf, in float r);

    // c = op_line(c, 0.);
    // c = op_fill(c, 0.);
    // c = op_line(c, 0.);
    // c = op_grow_fill(c, 0.01);
    // c = op_grow(c, 0.1);
    // c = op_grow_outer(c, 0.1);
    // c = op_grow_inner(c, 0.1);
    // c = op_fill_antialias(c, 0.);
    c = op_line_antialias(c, 0.);
    // c = op_fill_gradation(c, 0.);
    // c = op_line_gradation2(c, 0.);

    vec3 color = vec3(c);
    gl_FragColor = vec4(color, 1.);
}
