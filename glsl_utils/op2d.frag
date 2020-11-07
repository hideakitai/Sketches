// Imported from:
// https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm

// Making shapes rounded

// All the shapes above can be converted into rounded shapes by subtracting a
// constant from their distance function. That, effectivelly moves the
// isosurface (isopetimeter I guess) from the level zero to one of the outter
// rings, which naturally are rounded, as it can be seen in the yellow areas in
// all the images above. So, basically, for any shape defined by d(x,y) =
// sdf(x,y), one can make it sounded by computing d(x,y) = sdf(x,y) - r. You can
// learn more about this in this Youtube video.

float opRound(in float sdf, in float r) {
    return sdf - r;
}

// These are a few examples: rounded line, rounded triangle, rounded box and a
// rounded pentagon:

// Making shapes annular

// Similarly, shapes can be made annular (like a ring or the layers of an
// onion), but taking their absolute value and then substracting a constant from
// their field. So, for any shape defined by d(x,y) = sdf(x,y) compute d(x,y) =
// |sdf(x,y)| - r:

float opOnion(in float sdf, in float r) {
    return abs(sdf) - r;
}

// These are a few examples: annular rounded line, an annular triangle, an
// annular box and a annular pentagon:

// Other operations
// https://qiita.com/7CIT/items/ea3b41717323c83ecc35

float op_fill(in float sdf, in float r) {
    return step(0., r - sdf);
}

float op_line(in float sdf, in float r) {
    return step(abs(r - sdf), 0.005);
}

float op_grow_fill(in float sdf, in float r) {
    return r / sdf;
}

float op_grow(in float sdf, in float r) {
    return 0.01 / abs(sdf - r);
}

float op_grow_outer(in float sdf, in float r) {
    return 0.01 / (sdf - r);
}

float op_grow_inner(in float sdf, in float r) {
    return 0.01 / -(sdf - r);
}

float op_fill_antialias(in float sdf, in float r) {
    return smoothstep(0., 0.01, r - sdf);
}

float op_line_antialias(in float sdf, in float r) {
    return 1. - smoothstep(0.005, 0.015, abs(r - sdf));
}

float op_fill_gradation(in float sdf, in float r) {
    return smoothstep(0., 0.5, r - sdf);
}

float op_line_gradation(in float sdf, in float r) {
    return 1. - smoothstep(0.005, 0.5, abs(r - sdf));
}
