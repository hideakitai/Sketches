precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // fold
    vec2 p = abs(uv);

    vec3 color = vec3(p, 0.);
    gl_FragColor = vec4(color, 1.);
}
