precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

vec3 hsb2rgb(in vec3 c) {
    vec3 rgb = clamp(abs(mod(c.x * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
    rgb = rgb * rgb * (3.0 - 2.0 * rgb);
    return c.z * mix(vec3(1.0), rgb, c.y);
}

void main() {
    vec2 st = gl_FragCoord.xy / resolution.xy;
    float mx = mouse.x - step(0.5, mouse.x) * 2. * mod(mouse.x, 0.5);
    float my = mouse.y - step(0.5, mouse.y) * 2. * mod(mouse.y, 0.5);

    vec3 color_outer = hsb2rgb(vec3(mouse.x, 1., 1.));
    vec3 color_inner = hsb2rgb(vec3(fract(mouse.x + 0.5), 1., 1.));

    vec3 foreground = step(mx, st.x) * step(my, st.y) * vec3(1.);
    foreground *= step(mx, 1. - st.x) * step(my, 1. - st.y) * vec3(1.);

    vec3 background = abs(1. - foreground);

    vec3 color = background * color_outer + foreground * color_inner;

    gl_FragColor = vec4(color, 1.);
}
