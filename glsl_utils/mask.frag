// Imported from:
// https://qiita.com/7CIT/items/64c16cd1500fc25fee6d

float mask_radial(in vec2 p, in float rad1, in float rad2) {
    float a1 = mod(rad1, PI) / PI;        // -1 to 1
    float a2 = mod(abs(rad2), TAU) / TAU; //  0 to 1
    float a = fract(atan(p.y, p.x) / TAU + 1. - a1);
    return step(a, a2);
}

float mask_range(in float x, in float min, in float max) {
    return step(min, x) * step(x, max);
}

float mask_rect(in vec2 p, in vec2 p1, in vec2 p2) {
    return mask_range(p.x, p1.x, p2.x) * mask_range(p.y, p1.y, p2.y);
}
