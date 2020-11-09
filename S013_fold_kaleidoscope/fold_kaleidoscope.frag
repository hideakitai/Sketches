precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define FPS 60.0
#define BPM 120.0
#define LEN 32.0 // loop length
#define _beat (time * BPM / FPS)
#define beat (mod(_beat, LEN))

#ifndef saturate
#define saturate(x) clamp(x, 0., 1.)
#endif

float sdRect(vec2 p, vec2 b) {
    vec2 d = abs(p) - b;
    return max(d.x, d.y) + min(max(d.x, d.y), 0.0);
}

mat2 rot(float x) {
    return mat2(cos(x), sin(x), -sin(x), cos(x));
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, saturate(p - K.xxx), c.y);
}

// https://www.shadertoy.com/view/MdKfWR
float tex(in vec2 p, in float z) {
    vec2 q = (fract(p / 10.0) - 0.5) * 10.0; // scale
    // q = p;
    float d = 9999.0;
    // repeat fold + rotate
    for (int i = 0; i < 5; ++i) {
        // for (int i = 0; i < 1; ++i) {
        q = abs(q) - 0.5;                           // fold coordinate and move 0.5
        q *= rot(0.785398);                         // rotate coordinate 90 deg per loop
        q = abs(q) - 0.5;                           // fold coordinate and move 0.5
        q *= rot(z * 0.5);                          // rotate depending on time
        float k = sdRect(q, vec2(1.0, 0.55 + q.x)); // deform rect -> triangle about x-axis
        d = min(d, k);
    }
    // f = d;
    return d;
}

void main() {
    vec2 p = (gl_FragCoord.xy * 2.0 - resolution.xy) / min(resolution.x, resolution.y);

    float d = tex(p * 10., time);
    d = pow(1. / (1. - d), 2.); // scale
    d = smoothstep(0.2, 0.8, d);

    vec3 color = hsv2rgb(vec3((beat + 16.) / LEN, 1., 0.8));
    color = d * color;

    gl_FragColor = vec4(color, 1.0);
}
