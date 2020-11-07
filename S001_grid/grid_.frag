precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

void main() {
    vec2 st = gl_FragCoord.xy / resolution.xy; // normalize coordinate
    st.x *= resolution.x / resolution.y;       // scale depending on aspect
    float tiles = 10.;
    vec2 st_tiled = fract(st * tiles);

    vec3 color = vec3(0.0);
    color = vec3(st_tiled.x, st_tiled.y, abs(sin(time)));

    gl_FragColor = vec4(color, 1.0);
}
