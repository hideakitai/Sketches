// Imported from:
// https://qiita.com/7CIT/items/31b13830e11ba05959f0

float pattern_checkers(in vec2 p, in float n) {
    vec2 q = p * n;
    return mod(floor(q.x) + floor(q.y), 2.0);
}
