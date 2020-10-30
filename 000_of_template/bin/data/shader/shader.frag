#version 150

out vec4 color;

uniform float u_time;
uniform vec2 u_resolution;

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution.xy;
    float t = fract(u_time * 0.5);

    vec3 c = vec3(uv * t, 0.);
    color = vec4(c, 1.);
}
