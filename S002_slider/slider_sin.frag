precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

float rand(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    float tiles = 20.;
    vec2 st = gl_FragCoord.xy / resolution.xy;
    vec2 st_scaled = st * tiles;
    vec2 st_tile = fract(st_scaled);

    float x = sin(time + floor(st_scaled.y)) * 0.5 + 0.5;
    float y = sin(time + floor(st_scaled.x)) * 0.5 + 0.5;

    float cx = 1.0 - step(x, st.x);
    float cy = 1.0 - step(y, st.y);

    vec3 color = vec3(cx, cy, 0.);
    gl_FragColor = vec4(color, 1.);
}
