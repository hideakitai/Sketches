precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

void main() {
    vec2 st = gl_FragCoord.xy / resolution.xy;
    float t = fract(time);
    vec3 color = vec3(1.0 - step(t, fract(st.x)));
    gl_FragColor = vec4(color, 1.);
}
