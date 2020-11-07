precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592

float circle(vec2 center, float radius, vec2 st) {
    return 1.0 - step(radius, length(center - st));
}

float circle_line(vec2 center, float radius, float width, vec2 st) {
    return circle(center, radius + width / 2., st) -
           circle(center, radius - width / 2., st);
}

void main() {
    vec2 st = (gl_FragCoord.xy * 2. - resolution.xy) /
              min(resolution.x, resolution.y);

    float t = fract(time / 4.) * 2. * PI;
    vec2 p1 = 0.5 * vec2(cos(t), sin(t));
    vec3 color = vec3(circle(p1, 0.1, st));

    vec2 p2 = 0.5 * vec2(cos(t + PI), sin(t + PI));
    color += vec3(circle_line(p2, 0.1, 0.002, st));
    gl_FragColor = vec4(color, 1.);
}
