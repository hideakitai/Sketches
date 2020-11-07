precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

float metaball(vec2 center, float radius, vec2 st) {
    float dist = length(st - center);
    // if dist > size, value goes near to zero
    return pow(radius / dist, 2.0);
}

void main() {
    vec2 st = (gl_FragCoord.xy * 2. - resolution.xy) /
              min(resolution.x, resolution.y);

    float t = fract(time / 4.) * 2. * PI;
    vec2 p1 = 0.5 * vec2(cos(t), sin(t));
    vec2 p2 = 0.5 * vec2(cos(-t), sin(-t));

    vec3 color = vec3(0.);

    color += metaball(p1, 0.1, st);
    color += metaball(p2, 0.1, st);
    color += metaball(vec2(0.0), 0.5 * sin(time / 2.), st);

    gl_FragColor = vec4(vec3(color), 1.0);
}
