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
    float div_x = min(100., 3. * 1. / mouse.x);
    float div_y = min(100., 3. * 1. / mouse.y);
    float st_x = fract(st.x * div_x);
    float st_y = fract(st.y * div_y);
    float v_x = floor(st.x * div_x) / div_x;
    float v_y = floor(st.y * div_y) / div_y;

    vec3 color = hsb2rgb(vec3(v_x, v_y, 1.));

    gl_FragColor = vec4(color, 1.);
}
