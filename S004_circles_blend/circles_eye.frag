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

float noise(vec2 p) {
    vec2 ip = floor(p);
    vec2 u = fract(p);
    u = u * u * (3.0 - 2.0 * u);

    float res = mix(
        mix(rand(ip), rand(ip + vec2(1.0, 0.0)), u.x),
        mix(rand(ip + vec2(0.0, 1.0)), rand(ip + vec2(1.0, 1.0)), u.x), u.y);
    return res * res;
}

void main() {
    vec2 st = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);
    vec2 st_scaled = fract(st * 10.);
    vec2 st_tiled = floor(st * 10.);

    vec3 color = vec3(0.);
    float t = fract((time + 4.0 * rand(st_tiled)) / 4.0); // + noise(st);
    float r1 = 0.8 * pcurve(t, 10., 10.) * 1.;            // * rand(floor(st * 10.)); // + rand(floor(st * 10.));
    float r2 = 0.8 * pcurve(t, 10., 10.) * 1.;            // * rand(floor(st * 10.)); // + rand(floor(st * 10.));
    float c1 = circle_line(vec2(0.3, 0.3), r1, 0.15, fract(st * 10.));
    float c2 = circle_line(vec2(0.7, 0.7), r2, 0.15, fract(st * 10.));
    color = vec3(c1, c2, 0.);

    gl_FragColor = vec4(color, 1.);
}
