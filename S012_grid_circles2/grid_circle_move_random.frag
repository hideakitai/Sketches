precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
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

float opOnion(in float sdf, in float r) {
    return abs(sdf) - r;
}

float circle(in vec2 p, float r) {
    return length(p) - r;
}

// float noise(vec2 p) {
//     vec2 ip = floor(p);
//     vec2 u = fract(p);
//     u = u * u * (3.0 - 2.0 * u);

//     float res = mix(
//         mix(rand(ip), rand(ip + vec2(1.0, 0.0)), u.x),
//         mix(rand(ip + vec2(0.0, 1.0)), rand(ip + vec2(1.0, 1.0)), u.x), u.y);
//     return res * res;
// }

const float PI = 3.141592;
// Simplex 2D noise
//
vec3 permute(vec3 x) { return mod(((x * 34.0) + 1.0) * x, 289.0); }

float snoise(vec2 v) {
    const vec4 C = vec4(0.211324865405187, 0.366025403784439,
                        -0.577350269189626, 0.024390243902439);
    vec2 i = floor(v + dot(v, C.yy));
    vec2 x0 = v - i + dot(i, C.xx);
    vec2 i1;
    i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod(i, 289.0);
    vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));
    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy),
                            dot(x12.zw, x12.zw)),
                 0.0);
    m = m * m;
    m = m * m;
    vec3 x = 2.0 * fract(p * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);
    vec3 g;
    g.x = a0.x * x0.x + h.x * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

float gain(float x, float k) {
    float a = 0.5 * pow(2.0 * ((x < 0.5) ? x : 1.0 - x), k);
    return (x < 0.5) ? a : 1.0 - a;
}

void main() {
    vec2 uv = gl_FragCoord.xy / resolution.xy;
    uv.x *= resolution.x / resolution.y;
    float tiles = 5.;
    vec2 uv_tile = fract(uv * tiles);
    float rand_tile = floor(time / 4.);
    float type = mouse.x < 1. / 3. ? -1. : (mouse.x < 2. / 3. ? 0. : 1.);

    float radius = 0.8;

    // vec3 c1 = vec3(.81, .99, .95);
    // vec3 c2 = vec3(.67, .91, .94);
    // vec3 c3 = vec3(.65, .67, .93);
    // vec3 c4 = vec3(.65, .42, .76);

    vec3 c1 = vec3(.87, .96, .95);
    vec3 c2 = vec3(.87, .91, .95);
    vec3 c3 = vec3(.73, .73, .87);
    vec3 c4 = vec3(.53, .56, .80);

    vec4 c = vec4(0.);
    // 9 tiles around
    // for (int u = -1; u <= 1; ++u) {
    //     for (int v = -1; v <= 1; ++v) {
    for (int u = 0; u <= 0; ++u) {
        for (int v = 0; v <= 0; ++v) {
            vec2 p = uv_tile - vec2(u, v);
            vec2 idx = floor(uv * tiles + vec2(u, v));
            // change size
            // float r = radius * sin((time / 4. + 2. * rand(idx)) * 3.141592);
            float r = radius;
            float t = fract(time / 4.);
            float m = (t < 0.5) ? gain(t * 2., 6.) : 1. - gain((t - 0.5) * 2., 6.);
            m *= 0.5;

            float t_seed = floor(time / 4.);
            float seed = rand(idx + t_seed);
            vec2 dir = (seed < 0.25) ? vec2(1., 0.) : (seed < 0.5) ? vec2(0., 1.) : (seed < 0.75) ? vec2(-1., 0.) : vec2(0., -1.);

            float cs = 0.;
            cs = circle(p - vec2(0.5) + m * dir, 0.1);
            cs = opOnion(cs, 0.05);
            cs = op_fill(cs, 0.);

            float cs2 = 0.;
            cs2 = circle(p - vec2(0.5) + m * dir, 0.275);
            cs2 = opOnion(cs2, 0.05);
            cs2 = op_fill(cs2, 0.);

            float cs3 = 0.;
            cs3 = circle(p - vec2(0.5) + m * dir, 0.45);
            cs3 = opOnion(cs3, 0.05);
            cs3 = op_fill(cs3, 0.);

            cs += cs2 + cs3;

            vec3 base = vec3(1.);
            // base = (seed < 0.25) ? c1 : (seed < 0.5) ? c2 : (seed < 0.75) ? c3 : c4;

            c += vec4(base * cs * 0.35, 1.0);
        }
    }
    // c = vec4(op_grow(c.r, 0.), op_grow(c.g, 0.), op_grow(c.b, 0.), 1.0);

    c = vec4(vec3(c), 1.0);

    gl_FragColor = vec4(vec3(c), 0.01);
}
