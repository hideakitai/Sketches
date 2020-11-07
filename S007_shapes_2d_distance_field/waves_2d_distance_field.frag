precision highp float;
uniform vec2 mouse;           // mouse
uniform float time;           // time
uniform vec2 resolution;      // resolution
uniform sampler2D backbuffer; // prev scene

const float PI = 3.14;
const float PI2 = PI * 2.;

float op_line(in float sdf, in float r) {
    return step(abs(r - sdf), 0.005);
}

float sdSinWave(vec2 p) {
    return length(vec2(0, p.y + sin(p.x * PI)));
}

float sdSquareWaveForPolar(vec2 p) {
    float y = 2. * (step(0., sin(p.x * PI)) - .5);
    return length(vec2(0., p.y + y));
}

float sdSquareWave(vec2 p) {
    float y = -2. * (step(.5, fract(p.x * .5)) - .5);
    return length(vec2(0., p.y + y));
}

float sdSawWave(vec2 p) {
    float y = fract(-p.x * .5) * 2. - 1.;
    return length(vec2(0, p.y + y));
}

float sdTriangleWave(vec2 p) {
    float y = abs(2. * fract(p.x * .5 - .25) - 1.) * 2. - 1.;
    return length(vec2(0, p.y + y));
}

void main(void) {
    vec2 p = (gl_FragCoord.xy * 2.0 - resolution) / min(resolution.x, resolution.y);

    float c = 0.;
    c = sdSinWave(p);
    c = sdSquareWaveForPolar(p);
    c = sdSquareWave(p);
    c = sdSawWave(p);
    c = sdTriangleWave(p);

    c = op_line(c, 0.);

    gl_FragColor = vec4(vec3(c), 1.0);
}
