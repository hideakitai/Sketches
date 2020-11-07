precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

float circle(vec2 center, float radius, vec2 st) {
    return 1.0 - step(radius, length(center - st));
}

void main() {
    vec2 st = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);
    float t = fract(time / 1.) - 0.5;

    float circ = circle(vec2(0.), 0.5, st);

    float y = -st.x + 2. * t;
    float mask = step(y, st.y) - step(y + 0.4, st.y);

    vec3 color = vec3(circ);
    color *= vec3(mask);
    gl_FragColor = vec4(color, 1.);
}
