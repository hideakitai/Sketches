precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);
    // vec2 uv = gl_FragCoord.xy / resolution.xy;
    // uv.x *= resolution.y / resolution.x;

    float tiles = 10.;
    vec2 uv_tiled = fract(uv * tiles) - vec2(0.5);

    vec2 p = uv_tiled * 10. * (0.5 + sin(-3.141592 / 4. + time / 2.));
    float c = fract(length(p - vec2(0.0)));
    c = step(0.5, c);
    gl_FragColor = vec4(vec3(c), 1.);
}
