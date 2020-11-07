// Imported from:
// https://qiita.com/7CIT/items/31b13830e11ba05959f0

vec2 uv_even_center(in vec2 uv, in float n) {
    return mod(uv * n, 2.) - 1.;
}

vec2 uv_even_lb(in vec2 uv, in float n) {
    return mod((uv + 1.) * n, 2.) - 1.;
}

vec2 uv_even_rt(in vec2 uv, in float n) {
    return mod((uv - 1.) * n, 2.) - 1.;
}

vec2 uv_even_lt(in vec2 uv, in float n) {
    return mod((uv + vec2(1., -1.)) * n, 2.) - 1.;
}

vec2 uv_even_rb(in vec2 uv, in float n) {
    return mod((uv + vec2(-1., 1.)) * n, 2.) - 1.;
}

vec2 uv_odd_center(in vec2 uv, in float n) {
    return mod(uv * n + 1., 2.) - 1.;
}
