#include "math.glsl"

// Imported from:
// https : //www.iquilezles.org/www/articles/distfunctions/distfunctions.htm

// Intro
// After having posted about the basics of distance functions in several places
// (pouet, my blog, shadertoy, private emails, etc), I thought it might make
// sense to put these together in a centralized place. Here you will find the
// distance functions for basic primitives, plus the formulas for combining them
// together for building more complex shapes, as well as some distortion
// functions that you can use to shape your objects. Hopefully this will be
// useful for those rendering scenes with raymarching. You can see some of the
// results you can get by using these techniques in the raymarching distance
// fields article. Lastly, this article doesn't include lighting tricks, nor
// marching acceleration tricks or more advanced techniques as recursive
// primitives or fractals. In case you are looking for 2D SDF functions, you'll
// find them in the 2D Distance Functions page.

// Next to each primitives, modifier and operator function in this page you'll
// find a "exact" or "bound" note. This refers to the properties of the SDF that
// is generated or returned by the function. An "exact" SDF is one that retains
// all the qualities of a true SDF in Euclidean space - it really measures a
// distance exactly, meaning and its gradient always has length one. A "bound"
// SDF is no longer a true SDF (being pedantic) and only returns a lower bound
// to the real SDF, which can still be useful in certain scenarios. SDFs that
// are "exact" are generally desired over the "bound" ones because they work
// better with a bigger number of algorithms and techniques, and produce higher
// quality results. However some primitives (like the ellipsoid) or operators
// (like the smooth minimum here) cannot be "exact" because the very mathematics
// that describe them prevent it. In those cases, I propose a "bound" version
// and mark it so to prevent confusion. If a primitive that has an "exact"
// implementation can also be approximated by a "bound" version and sometimes I
// sometimes document that one too, but not too often because while locally
// faster than their "exact" counterpart they often backfire in the form of a
// global performance hit (ironically) due to their poorer quality in measuring
// distances.

// Primitives

// All primitives are centered at the origin. You will have to transform the
// point to get arbitrarily rotated, translated and scaled objects (see below).
// Many of these primtives below use dot2() or ndot(), which I list here quickly
// before the primitives:

// float dot2(in vec2 v) { return dot(v, v); }
// float dot2(in vec3 v) { return dot(v, v); }
// float ndot(in vec2 a, in vec2 b) { return a.x * b.x - a.y * b.y; }

// Lastly, you have working sample code of all of these primitives here:
// https://www.shadertoy.com/view/Xds3zN

// Sphere - exact

float sdSphere(vec3 p, float s) { return length(p) - s; }

// Box - exact

// Youtube Tutorial on formula derivation:
// https://www.youtube.com/watch?v=62-pRVZuS5c

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Round Box - exact

float sdRoundBox(vec3 p, vec3 b, float r) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}

// Bounding Box - exact

float sdBoundingBox(vec3 p, vec3 b, float e) {
    p = abs(p) - b;
    vec3 q = abs(p + e) - e;
    return min(min(length(max(vec3(p.x, q.y, q.z), 0.0)) +
                       min(max(p.x, max(q.y, q.z)), 0.0),
                   length(max(vec3(q.x, p.y, q.z), 0.0)) +
                       min(max(q.x, max(p.y, q.z)), 0.0)),
               length(max(vec3(q.x, q.y, p.z), 0.0)) +
                   min(max(q.x, max(q.y, p.z)), 0.0));
}

// Torus - exact

float sdTorus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

// Capped Torus - exact

float sdCappedTorus(in vec3 p, in vec2 sc, in float ra, in float rb) {
    p.x = abs(p.x);
    float k = (sc.y * p.x > sc.x * p.y) ? dot(p.xy, sc) : length(p.xy);
    return sqrt(dot(p, p) + ra * ra - 2.0 * ra * k) - rb;
}

// Link - exact

float sdLink(vec3 p, float le, float r1, float r2) {
    vec3 q = vec3(p.x, max(abs(p.y) - le, 0.0), p.z);
    return length(vec2(length(q.xy) - r1, q.z)) - r2;
}

// Infinite Cylinder - exact

float sdCylinder(vec3 p, vec3 c) { return length(p.xz - c.xy) - c.z; }

// Cone - exact

float sdCone(in vec3 p, in vec2 c, float h) {
    // c is the sin/cos of the angle, h is height
    // Alternatively pass q instead of (c,h),
    // which is the point at the base in 2D
    vec2 q = h * vec2(c.x / c.y, -1.0);

    vec2 w = vec2(length(p.xz), p.y);
    vec2 a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    vec2 b = w - q * vec2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    float k = sign(q.y);
    float d = min(dot(a, a), dot(b, b));
    float s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// Cone - bound (not exact!)

float sdCone(vec3 p, vec2 c, float h) {
    float q = length(p.xz);
    return max(dot(c.xy, vec2(q, p.y)), -h - p.y);
}

// Infinite Cone - exact

float sdCone(vec3 p, vec2 c) {
    // c is the sin/cos of the angle
    vec2 q = vec2(length(p.xz), -p.y);
    float d = length(q - c * max(dot(q, c), 0.0));
    return d * ((q.x * c.y - q.y * c.x < 0.0) ? -1.0 : 1.0);
}

// Plane - exact

float sdPlane(vec3 p, vec3 n, float h) {
    // n must be normalized
    return dot(p, n) + h;
}

// Hexagonal Prism - exact

float sdHexPrism(vec3 p, vec2 h) {
    const vec3 k = vec3(-0.8660254, 0.5, 0.57735);
    p = abs(p);
    p.xy -= 2.0 * min(dot(k.xy, p.xy), 0.0) * k.xy;
    vec2 d = vec2(length(p.xy - vec2(clamp(p.x, -k.z * h.x, k.z * h.x), h.x)) *
                      sign(p.y - h.x),
                  p.z - h.y);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Triangular Prism - bound

float sdTriPrism(vec3 p, vec2 h) {
    vec3 q = abs(p);
    return max(q.z - h.y, max(q.x * 0.866025 + p.y * 0.5, -p.y) - h.x * 0.5);
}

// Capsule / Line - exact

float sdCapsule(vec3 p, vec3 a, vec3 b, float r) {
    vec3 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// Capsule / Line - exact

float sdVerticalCapsule(vec3 p, float h, float r) {
    p.y -= clamp(p.y, 0.0, h);
    return length(p) - r;
}

// Capped Cylinder - exact

float sdCappedCylinder(vec3 p, float h, float r) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - vec2(h, r);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Capped Cylinder - exact

float sdCappedCylinder(vec3 p, vec3 a, vec3 b, float r) {
    vec3 ba = b - a;
    vec3 pa = p - a;
    float baba = dot(ba, ba);
    float paba = dot(pa, ba);
    float x = length(pa * baba - ba * paba) - r * baba;
    float y = abs(paba - baba * 0.5) - baba * 0.5;
    float x2 = x * x;
    float y2 = y * y * baba;
    float d = (max(x, y) < 0.0)
                  ? -min(x2, y2)
                  : (((x > 0.0) ? x2 : 0.0) + ((y > 0.0) ? y2 : 0.0));
    return sign(d) * sqrt(abs(d)) / baba;
}

// Rounded Cylinder - exact

float sdRoundedCylinder(vec3 p, float ra, float rb, float h) {
    vec2 d = vec2(length(p.xz) - 2.0 * ra + rb, abs(p.y) - h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rb;
}

// Capped Cone - exact

float sdCappedCone(vec3 p, float h, float r1, float r2) {
    vec2 q = vec2(length(p.xz), p.y);
    vec2 k1 = vec2(r2, h);
    vec2 k2 = vec2(r2 - r1, 2.0 * h);
    vec2 ca = vec2(q.x - min(q.x, (q.y < 0.0) ? r1 : r2), abs(q.y) - h);
    vec2 cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot2(k2), 0.0, 1.0);
    float s = (cb.x < 0.0 && ca.y < 0.0) ? -1.0 : 1.0;
    return s * sqrt(min(dot2(ca), dot2(cb)));
}

// Capped Cone - exact

float sdCappedCone(vec3 p, vec3 a, vec3 b, float ra, float rb) {
    float rba = rb - ra;
    float baba = dot(b - a, b - a);
    float papa = dot(p - a, p - a);
    float paba = dot(p - a, b - a) / baba;
    float x = sqrt(papa - paba * paba * baba);
    float cax = max(0.0, x - ((paba < 0.5) ? ra : rb));
    float cay = abs(paba - 0.5) - 0.5;
    float k = rba * rba + baba;
    float f = clamp((rba * (x - ra) + paba * baba) / k, 0.0, 1.0);
    float cbx = x - ra - f * rba;
    float cby = paba - f;
    float s = (cbx < 0.0 && cay < 0.0) ? -1.0 : 1.0;
    return s * sqrt(min(cax * cax + cay * cay * baba,
                        cbx * cbx + cby * cby * baba));
}

// Solid Angle - exact

float sdSolidAngle(vec3 p, vec2 c, float ra) {
    // c is the sin/cos of the angle
    vec2 q = vec2(length(p.xz), p.y);
    float l = length(q) - ra;
    float m = length(q - c * clamp(dot(q, c), 0.0, ra));
    return max(l, m * sign(c.y * q.x - c.x * q.y));
}

// Round cone - exact

float sdRoundCone(vec3 p, float r1, float r2, float h) {
    vec2 q = vec2(length(p.xz), p.y);

    float b = (r1 - r2) / h;
    float a = sqrt(1.0 - b * b);
    float k = dot(q, vec2(-b, a));

    if (k < 0.0)
        return length(q) - r1;
    if (k > a * h)
        return length(q - vec2(0.0, h)) - r2;

    return dot(q, vec2(a, b)) - r1;
}

// Round Cone - exact

float sdRoundCone(vec3 p, vec3 a, vec3 b, float r1, float r2) {
    // sampling independent computations (only depend on shape)
    vec3 ba = b - a;
    float l2 = dot(ba, ba);
    float rr = r1 - r2;
    float a2 = l2 - rr * rr;
    float il2 = 1.0 / l2;

    // sampling dependant computations
    vec3 pa = p - a;
    float y = dot(pa, ba);
    float z = y - l2;
    float x2 = dot2(pa * l2 - ba * y);
    float y2 = y * y * l2;
    float z2 = z * z * l2;

    // single square root!
    float k = sign(rr) * rr * rr * x2;
    if (sign(z) * a2 * z2 > k)
        return sqrt(x2 + z2) * il2 - r2;
    if (sign(y) * a2 * y2 < k)
        return sqrt(x2 + y2) * il2 - r1;
    return (sqrt(x2 * a2 * il2) + y * rr) * il2 - r1;
}

// Ellipsoid - bound (not exact!)

float sdEllipsoid(vec3 p, vec3 r) {
    float k0 = length(p / r);
    float k1 = length(p / (r * r));
    return k0 * (k0 - 1.0) / k1;
}

// Rhombus - exact

float sdRhombus(vec3 p, float la, float lb, float h, float ra) {
    p = abs(p);
    vec2 b = vec2(la, lb);
    float f = clamp((ndot(b, b - 2.0 * p.xz)) / dot(b, b), -1.0, 1.0);
    vec2 q = vec2(length(p.xz - 0.5 * b * vec2(1.0 - f, 1.0 + f)) *
                          sign(p.x * b.y + p.z * b.x - b.x * b.y) -
                      ra,
                  p.y - h);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0));
}

// Octahedron - exact

float sdOctahedron(vec3 p, float s) {
    p = abs(p);
    float m = p.x + p.y + p.z - s;
    vec3 q;
    if (3.0 * p.x < m)
        q = p.xyz;
    else if (3.0 * p.y < m)
        q = p.yzx;
    else if (3.0 * p.z < m)
        q = p.zxy;
    else
        return m * 0.57735027;

    float k = clamp(0.5 * (q.z - q.y + s), 0.0, s);
    return length(vec3(q.x, q.y - s + k, q.z - k));
}

// Octahedron - bound (not exact)

float sdOctahedron(vec3 p, float s) {
    p = abs(p);
    return (p.x + p.y + p.z - s) * 0.57735027;
}

// Pyramid - exact

float sdPyramid(vec3 p, float h) {
    float m2 = h * h + 0.25;

    p.xz = abs(p.xz);
    p.xz = (p.z > p.x) ? p.zx : p.xz;
    p.xz -= 0.5;

    vec3 q = vec3(p.z, h * p.y - 0.5 * p.x, h * p.x + 0.5 * p.y);

    float s = max(-q.x, 0.0);
    float t = clamp((q.y - 0.5 * p.z) / (m2 + 0.25), 0.0, 1.0);

    float a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    float b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) +
              (q.y - m2 * t) * (q.y - m2 * t);

    float d2 = min(q.y, -q.x * m2 - q.y * 0.5) > 0.0 ? 0.0 : min(a, b);

    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -p.y));
}

// Triangle - exact

float udTriangle(vec3 p, vec3 a, vec3 b, vec3 c) {
    vec3 ba = b - a;
    vec3 pa = p - a;
    vec3 cb = c - b;
    vec3 pb = p - b;
    vec3 ac = a - c;
    vec3 pc = p - c;
    vec3 nor = cross(ba, ac);

    return sqrt(
        (sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) +
             sign(dot(cross(ac, nor), pc)) <
         2.0)
            ? min(min(dot2(ba * clamp(dot(ba, pa) / dot2(ba), 0.0, 1.0) - pa),
                      dot2(cb * clamp(dot(cb, pb) / dot2(cb), 0.0, 1.0) - pb)),
                  dot2(ac * clamp(dot(ac, pc) / dot2(ac), 0.0, 1.0) - pc))
            : dot(nor, pa) * dot(nor, pa) / dot2(nor));
}

// Quad - exact

float udQuad(vec3 p, vec3 a, vec3 b, vec3 c, vec3 d) {
    vec3 ba = b - a;
    vec3 pa = p - a;
    vec3 cb = c - b;
    vec3 pb = p - b;
    vec3 dc = d - c;
    vec3 pc = p - c;
    vec3 ad = a - d;
    vec3 pd = p - d;
    vec3 nor = cross(ba, ad);

    return sqrt(
        (sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) +
             sign(dot(cross(dc, nor), pc)) + sign(dot(cross(ad, nor), pd)) <
         3.0)
            ? min(min(min(dot2(ba * clamp(dot(ba, pa) / dot2(ba), 0.0, 1.0) -
                               pa),
                          dot2(cb * clamp(dot(cb, pb) / dot2(cb), 0.0, 1.0) -
                               pb)),
                      dot2(dc * clamp(dot(dc, pc) / dot2(dc), 0.0, 1.0) - pc)),
                  dot2(ad * clamp(dot(ad, pd) / dot2(ad), 0.0, 1.0) - pd))
            : dot(nor, pa) * dot(nor, pa) / dot2(nor));
}
