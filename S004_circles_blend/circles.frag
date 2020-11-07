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

float pcurve(float x, float a, float b) {
    float k = pow(a + b, a + b) / (pow(a, a) * pow(b, b));
    return k * pow(x, a) * pow(1.0 - x, b);
}

float rand(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 st = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    float color = 0.;
    const float num = 200.;
    float t = fract(time / 4.0);
    float r = rand(vec2(1.) * floor(time / 4.0));
    for (float i = 0.; i < num; ++i) {
        float rad = 2. * PI * i / num;
        float radius = 5. * (1. - r) * pcurve(t, 13., 13.);
        // float radius = 5. * (1. - r) * sin(t * PI);
        vec2 c = r * vec2(cos(rad), sin(rad));
        color += circle_line(c, radius, 0.001, st);
    }
    gl_FragColor = vec4(vec3(color), 1.);
}
