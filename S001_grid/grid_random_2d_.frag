precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

float rand(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    float tiles = 10.;
    vec2 st = gl_FragCoord.xy / resolution.xy;
    vec2 st_tiled = st * tiles;
    float t_scale = 0.5;             // interval 2 sec
    float t = fract(time * t_scale); // 0.0 - 2.0
    vec2 seed = floor(st_tiled) + floor(time * t_scale);

    float rand_scale = 10.;                     // to generate random floor()
    float rad = floor(rand(seed) * rand_scale); // generate int 0 - 10
    rad *= PI / 2.;                             // fix phase to sin() or cos() is 0 and the other is one
    vec2 rand_tile = vec2(sin(rad), cos(rad));  // one is 1 and the other is 0

    vec3 color = vec3(abs(rand_tile), 0.); // rand_tile is [-1, 1]
    gl_FragColor = vec4(color, 1.);
}
