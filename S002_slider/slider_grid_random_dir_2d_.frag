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
    vec2 st_s = gl_FragCoord.xy / resolution.xy * tiles;
    vec2 st_t = fract(st_s);
    float t_scale = 0.5;             // interval 2 sec
    float t = fract(time * t_scale); // 0.0 - 2.0
    vec2 seed =
        0.5 + floor(st_s) + floor(time * t_scale); // change seed every 2 sec
    float rand_scale = 10.;                        // to generate random floor()
    float rad = floor(rand(seed) * rand_scale);    // generate int 0 - 10
    rad *= PI / 2.;                                // fix phase to sin() or cos() is 0 and the other is one
    vec2 rand_tile = vec2(sin(rad), cos(rad));     // one is 1 and the other is 0
    rand_tile *= 1.0;                              // scale value
    rand_tile *= smoothstep(0.1, 0.4, t);          // ease in
    rand_tile *= smoothstep(0.9, 0.6, t);          // ease out

    vec2 st_dir = st_t;
    if (rand_tile.x < 0.0 || rand_tile.y < 0.0) // if rand_tile is minus,
        st_dir = 1.0 - st_t;                    // invert direction

    // draw if st is inside rand_tile
    vec2 slider = 1.0 - step(abs(rand_tile), st_dir);

    // vec3 color = vec3(slider, 0.0);
    vec3 color = vec3(slider, 0.);
    gl_FragColor = vec4(color, 1.);
}
