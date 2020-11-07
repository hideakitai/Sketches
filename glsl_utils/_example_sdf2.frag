#include "../glsl_utils/op2d.frag"
#include "../glsl_utils/sdf2d.frag"

precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

void main() {
    vec2 uv = (gl_FragCoord.xy * 2. - resolution) / min(resolution.x, resolution.y);

    // create sdf
    float c = sdCircle(uv, 0.5);
    // float c = sdRect(uv, vec2(0.2));

    // set distance to make shape from sdf
    // c = opFill(c, 0.);
    c = opLine(c, 0.5);
    // c = opOnion(c, 0.);
    // c = opFill(c, 0.03);
    vec3 color = vec3(c);
    gl_FragColor = vec4(color, 1.);
}
