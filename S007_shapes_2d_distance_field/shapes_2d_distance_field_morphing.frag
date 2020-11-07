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
    float tiles = 2.;
    vec2 uv_scaled = gl_FragCoord.xy / resolution.xy * tiles;
    vec2 uv_tiled = fract(uv_scaled) + vec2(-0.5);
    float idx = floor(uv_scaled.x) + tiles * floor(uv_scaled.y);

    float t = fract(time * 0.5);
    float state = floor(mod(time * 0.5, 4.));

    float c1 = circle(uv_tiled, 0.2);
    float c2 = square(uv_tiled, 0.2);
    float c3 = polygon(uv_tiled * 2., 3.) - 0.5;
    float c4 = rect(uv_tiled, vec2(0.3, 0.15));

    float co = 0.;
    if (idx == 0.) {
        if (state == 0.)
            co = mix(c1, c2, t);
        else if (state == 1.)
            co = mix(c2, c3, t);
        else if (state == 2.)
            co = mix(c3, c4, t);
        else if (state == 3.)
            co = mix(c4, c1, t);
    } else if (idx == 1.) {
        if (state == 1.)
            co = mix(c1, c2, t);
        else if (state == 2.)
            co = mix(c2, c3, t);
        else if (state == 3.)
            co = mix(c3, c4, t);
        else if (state == 0.)
            co = mix(c4, c1, t);
    } else if (idx == 2.) {
        if (state == 2.)
            co = mix(c1, c2, t);
        else if (state == 3.)
            co = mix(c2, c3, t);
        else if (state == 0.)
            co = mix(c3, c4, t);
        else if (state == 1.)
            co = mix(c4, c1, t);
    } else if (idx == 3.) {
        if (state == 3.)
            co = mix(c1, c2, t);
        else if (state == 0.)
            co = mix(c2, c3, t);
        else if (state == 1.)
            co = mix(c3, c4, t);
        else if (state == 2.)
            co = mix(c4, c1, t);
    }

    co = op_fill_antialias(co, 0.);

    vec3 color = vec3(co);
    gl_FragColor = vec4(color, 1.);
}
