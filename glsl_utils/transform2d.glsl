// other transformations:
// https://www.shadertoy.com/view/MlGGz3

mat2 opScale(in vec2 s) {
    return mat2(s.x, 0.0, 0.0, s.y);
}

mat2 opRotate(in float rad) {
    float c = cos(rad);
    float s = sin(rad);
    return mat2(c, -s, s, c);
}

mat2 scale(in vec2 s) {
    return mat2(s.x, 0., 0., s.y);
}

mat2 sheer_x(in float a) {
    return mat2(1., tan(a), 0., 1.);
}
mat2 sheer_y(in float a) {
    return mat2(1., 0., tan(a), 1.);
}
