precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

float rand(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 st = gl_FragCoord.xy / resolution.xy;
    float tiles = 10.;
    vec2 st_tiled = st * tiles;

    float t_scale = 0.5;             // interval 2 sec
    float t = fract(time * t_scale); // 0.0 - 2.0

    // change seed every 2 sec
    vec2 seed = floor(st_tiled) + floor(time * t_scale);
    vec3 color = vec3(rand(seed));
    gl_FragColor = vec4(color, 1.);
}
