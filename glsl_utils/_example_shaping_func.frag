#include "shaping_funcs.glsl"

// uniform vec2 u_resolution;
// uniform vec2 u_mouse;
// uniform float u_time;
#define u_resolution iResolution
#define u_mouse iMouse
#define u_time iTime

void main() {
    vec2 st = gl_FragCoord.xy / u_resolution.xy;

    //  Function from Iñigo Quiles
    //  http://www.iquilezles.org/www/articles/functions/functions.htm

    float y = 0.;
    // y = smoothstep(0.2, 0.8, st.x);
    // y = almost_identity(st.x, 0.05, 0.3);
    // y = almost_unit_identity(st.x);
    // y = almost_identity(st.x, 0.05);
    // y = exp_impulse(st.x, 10.);
    // y = exp_sustained_impulse(st.x, 8., 0.1) / 2.;
    // y = quad_impulse(st.x, 100.);
    // y = poly_impulse(100., 5., st.x);
    // y = cubic_pulse(0.3, 0.2, st.x);
    // y = exp_step(st.x, 10., 5.);
    // y = gain(st.x, 10.);
    // y = parabola(st.x, 15.);
    y = pcurve(st.x, 13., 13.);
    // y = 0.5 * sinc(st.x, 10.) + 0.3;

    float pct = plot(st, y);
    vec3 color = vec3(y);
    color = (1. - pct) * color + pct * vec3(0., 1., 0.);

    gl_FragColor = vec4(color, 1.);
}
