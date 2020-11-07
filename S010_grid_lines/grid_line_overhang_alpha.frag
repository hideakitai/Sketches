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

float sdLine(in vec2 p, in vec2 v1, in vec2 v2) {
    vec2 v = v2 - v1;
    vec2 d = normalize(v); //direction
    p -= v1;
    return 1. - (1. - abs(cross(p, d))) * step(0., length(v) * .5 - abs(dot(p - v * .5, d)));
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

float circle(in vec2 p, float r) {
    return length(p) - r;
}

void main() {
    // vec2 uv = (gl_FragCoord.xy * 2. - resolution) / min(resolution.x, resolution.y);
    vec2 uv = gl_FragCoord.xy / resolution.xy;
    uv.x *= resolution.x / resolution.y;
    float tiles = 20.;
    vec2 uv_tile = fract(uv * tiles);
    float rand_tile = floor(time / 4.);
    float type = mouse.x < 1. / 3. ? -1. : (mouse.x < 2. / 3. ? 0. : 1.);

    float p_min = -0.0;
    float p_max = 1.0;
    vec2 plb = vec2(p_min, p_min);
    vec2 plt = vec2(p_min, p_max);
    vec2 prt = vec2(p_max, p_max);
    vec2 prb = vec2(p_max, p_min);

    vec4 c = vec4(0.);
    // 9 tiles around
    for (int u = -1; u <= 1; ++u) {
        for (int v = -1; v <= 1; ++v) {
            vec2 p = uv_tile - vec2(u, v);
            vec2 idx = floor(uv * tiles + vec2(u, v));
            // change direction
            float sel = rand(idx + rand_tile);
            float dir = sel > 0.5 ? 1. : 0.;
            vec2 p1 = dir == 1. ? plb : plt;
            vec2 p2 = dir == 1. ? prt : prb;
            float w = dir == 1. ? 0.2 : 0.1;

            float cs = 0.;

            // cs = (type == -1.) ? sdLineBat(p, p1, p2) : (type == 0. ? sdLineRound(p, p1, p2) : sdLineArrow(p, p1, p2));
            cs = sdLineRound(p, p1, p2);
            // cs = sdLineArrow(p, p1, p2);

            cs = op_fill(cs, w);
            vec3 base = (dir == 1.) ? vec3(.671875, .90234375, .93359375) : vec3(.6484375, .671875, .921875);
            // c += vec4(vec3(base * cs), 0.2);
            c += vec4(base * cs * 0.5, 1.0);
        }
    }

    gl_FragColor = vec4(vec3(c), 0.01);
}
