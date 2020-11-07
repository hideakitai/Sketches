precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

float circle(vec2 center, float radius, vec2 st) {
    return 1.0 - step(radius, length(center - st));
}

float rand(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 st = gl_FragCoord.xy / min(resolution.x, resolution.y);
    float tiles = 10.;
    vec2 st_tiled = fract(st * tiles);
    float t = fract(time / 1.);

    float circ = circle(vec2(0.5), 0.3, st_tiled);

    float y = -st_tiled.x + 2. * t + rand(floor(st * tiles));
    y = mod(y, 2.);
    float mask = step(y, st_tiled.y) - step(y + 0.4, st_tiled.y);

    float y2 = fract(st.x + 2. * fract(time / 4.) - 1.);
    float y22 = y2 + 0.2;
    float mask2 = step(y2, st.y) - step(y22, st.y);
    // if top area is overflow, add such area to bottom
    // if (y22 > 1.)
    //     mask2 += 1.0 - step(fract(y2 + 0.4), st.y);
    mask2 += step(1., y22) * (1.0 - step(fract(y22), st.y));

    vec3 color = vec3(circ);
    color *= vec3(mask);
    color *= vec3(mask2);

    gl_FragColor = vec4(color, 1.);
}
