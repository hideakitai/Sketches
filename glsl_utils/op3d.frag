// Primitive alterations
// Once we have the basic primitives, it's possible to apply some simple
// operations that change their shape while still retaining exact an euclidean
// metric to them, which is an important property since SDFs with undistorted
// euclidean metric allow for faster ray marchine.

// Elongation - exact

// Elongating is a useful way to construct new shapes. It basically splits a
// primitive in two (four or eight), moves the pieces apart and and connects
// them. It is a perfect distance preserving operation, it does not introduce
// any artifacts in the SDF. Some of the basic primitives above use this
// technique. For example, the Capsule is an elongated Sphere along an axis
// really. You can find code here: https://www.shadertoy.com/view/Ml3fWj

float opElongate(in sdf3d primitive, in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return primitive(q);
}

float opElongate(in sdf3d primitive, in vec3 p, in vec3 h) {
    vec3 q = abs(p) - h;
    return primitive(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// The reason I provide to implementations is the following. For 1D elongations,
// the first function works perfectly and gives exact exterior and interior
// distances. However, the first implementation produces a small core of zero
// distances inside the volume for 2D and 3D elongations. Depending on your
// application that might be a problem. One way to create exact interior
// distances all the way to the very elongated core of the volume, is the
// following, which is in languages like GLSL that don't have function pointers
// or lambdas need to be implemented a bit differently (check the code linked
// about in Shadertoy to see one example).

// Rounding - exact

// Rounding a shape is as simple as subtracting some distance (jumping to a
// different isosurface). The rounded box above is an example, but you can apply
// it to cones, hexagons or any other shape like the cone in the image below. If
// you happen to be interested in preserving the overall volume of the shape,
// most of the times it's pretty easy to shrink the source primitive by the same
// amount we are rounding it by. You can find code here:
// https://www.shadertoy.com/view/Mt3BDj

float opRound(in sdf3d primitive, float rad) { return primitive(p) - rad }

// Onion - exact

// For carving interiors or giving thickness to primitives, without performing
// expensive boolean operations (see below) and without distorting the distance
// field into a bound, one can use "onioning". You can use it multiple times to
// create concentric layers in your SDF. You can find code here:
// https://www.shadertoy.com/view/MlcBDj

float opOnion(in float sdf, in float thickness) { return abs(sdf) - thickness; }

// Revolution and extrusion from 2D - exact

// Generating 3D volumes from 2D shapes has many advantages. Assuming the 2D
// shape defines exact distances, the resulting 3D volume is exact and way often
// less intensive to evaluate than when produced from boolean operations on
// other volumes. Two of the most simplest way to make volumes our of flat
// shapes is to use extrusion and revolution (generalizations of these are easy
// to build, but we we'll keep simple here): You can find code here:
// https://www.shadertoy.com/view/4lyfzw

float opExtrusion(in vec3 p, in sdf2d primitive, in float h) {
    float d = primitive(p.xy) vec2 w = vec2(d, abs(p.z) - h);
    return min(max(w.x, w.y), 0.0) + length(max(w, 0.0));
}

float opRevolution(in vec3 p, in sdf2d primitive, float o) {
    vec2 q = vec2(length(p.xz) - o, p.y);
    return primitive(q)
}

// Change of Metric - bound

// Most of these functions can be modified to use other norms than the
// euclidean. By replacing length(p), which computes (x2+y2+z2)1/2 by
// (xn+yn+zn)1/n one can get variations of the basic primitives that have
// rounded edges rather than sharp ones. I do not recommend this technique
// though, since these primitives require more raymarching steps until an
// intersection is found than euclidean primitives. Since they only give a bound
// to the real SDF, this kind of primitive alteration also doesn't play well
// with shadows and occlusion algorithms that rely on true SDFs for measuring
// distance to occluders. You can find the code here:
// https://www.shadertoy.com/view/ltcfDj

float length2(vec3 p) {
    p = p * p;
    return sqrt(p.x + p.y + p.z);
}

float length6(vec3 p) {
    p = p * p * p;
    p = p * p;
    return pow(p.x + p.y + p.z, 1.0 / 6.0);
}

float length8(vec3 p) {
    p = p * p;
    p = p * p;
    p = p * p;
    return pow(p.x + p.y + p.z, 1.0 / 8.0);
}

// Primitive combinations
// Sometimes you cannot simply elongate, round or onion a primitive, and you
// need to combine, carve or intersect basic primitives. Given the SDFs d1 and
// d2 of two primitives, you can use the following operators to combine
// together.

// Union, Subtraction, Intersection - exact/bound, bound, bound

// These are the most basic combinations of pairs of primitives you can do. They
// correspond to the basic boolean operations. Please note that only the Union
// of two SDFs returns a true SDF, not the Subtraction or Intersection. To make
// it more subtle, this is only true in the exterior of the SDF (where distances
// are positive) and not in the interior. You can learn more about this and how
// to work around it in the article "Interior Distances". Also note that
// opSubtraction() is not commutative and depending on the order of the operand
// it will produce different results.

float opUnion(float d1, float d2) { min(d1, d2); }

float opSubtraction(float d1, float d2) { return max(-d1, d2); }

float opIntersection(float d1, float d2) { return max(d1, d2); }

// Smooth Union, Subtraction and Intersection - bound, bound, bound

// Blending primitives is a really powerful tool - it allows to construct
// complex and organic shapes without the geometrical semas that normal boolean
// operations produce. There are many flavors of such operations, but the basic
// ones try to replace the min() and max() functions used in the opUnion,
// opSubstraction and opIntersection above with smooth versions. They all accept
// an extra parameter called k that defines the size of the smooth transition
// between the two primitives. It is given in actual distance units. You can
// find more details in the smooth minimum article article in this same site.
// You can code here: https://www.shadertoy.com/view/lt3BW2

float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

float opSmoothSubtraction(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

float opSmoothIntersection(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}

// Positioning
// Placing primitives in different locations and orientations in space is a
// fundamental operation in designing SDFs. While rotations, uniform scaling and
// translations are exact operations, non-uniform scaling distorts the euclidean
// spaces and can only be bound. Therefore I do not include it here.

// Rotation/Translation - exact

// Since rotations and translation don't compress nor dilate space, all we need
// to do is simply to transform the point being sampled with the inverse of the
// transformation used to place an object in the scene. This code below assumes
// that transform encodes only a rotation and a translation (as a 3x4 matrix for
// example, or as a quaternion and a vector), and that it does not contain any
// scaling factors in it.

vec3 opTx(in vec3 p, in transform t, in sdf3d primitive) {
    return primitive(invert(t) * p);
}

// Scale - exact

// Scaling an obect is slightly more tricky since that compresses/dilates
// spaces, so we have to take that into account on the resulting distance
// estimation. Still, it's not difficult to perform:

float opScale(in vec3 p, in float s, in sdf3d primitive) {
    return primitive(p / s) * s;
}

// Symmetry - bound and exact

// Symmetry is useful, since many things around us are symmetric, from humans,
// animals, vehicles, instruments, furniture, ... Oftentimes, one can take
// shortcuts and only model half or a quarter of the desired shape, and get it
// duplicated automatically by using the absolute value of the domain
// coordinates before evaluation. For example, in the image below, there's a
// single object evaluation instead of two. This is a great savings in
// performance. You have to be aware however that the resuluting SDF might not
// be an exact SDF but a bound, if the object you are mirroring crosses the
// mirroring plane.

float opSymX(in vec3 p, in sdf3d primitive) {
    p.x = abs(p.x);
    return primitive(p);
}

float opSymXZ(in vec3 p, in sdf3d primitive) {
    p.xz = abs(p.xz);
    return primitive(p);
}

// Infinite Repetition

// Domain repetition is a very useful operator, since it allows you to create
// infinitely many primitives with a single object evaluator and without
// increasing the memory footprint of your application. The code below shows how
// to perform the operation in the simplest way:

float opRep(in vec3 p, in vec3 c, in sdf3d primitive) {
    vec3 q = mod(p + 0.5 * c, c) - 0.5 * c;
    return primitive(q);
}

// In this code c is the repetition period (which can be different in each
// coordinate direction). This will work great for primitives that have a
// bounding box smaller than half the repetition period. If the object is big,
// you will need to check the 7 neighboring repetitions (in 3D, 3 in 2D) to
// check for closest neighbors, just as you usually do in a
// voronoi/Worley/cellular construction. You have an example of this in action
// in the following image where all of the grass field is made from a single
// blade which repeates infinitely in the X and Z directions with the code
// above. (A link to the real time animation and code for the image is right
// below the image).

// https://www.shadertoy.com/view/4tByz3

// Finite Repetition

// Infinite domain repetition is great, but sometimes you only need a few copies
// or instances of a given SDF, not infinite. A frequently seen but suboptimal
// solution is to generate infinite copies and then clip the unwanted areas away
// with a box SDF. This is not ideal because the resulting SDF is not a real SDF
// but just a bound, since clipping through max() only produces a bound. A much
// better approach is to clamp the indices of the instances instead of the SDF,
// and let a correct SDF emerge from the truncated/clamped indices:

vec3 opRepLim(in vec3 p, in float c, in vec3 l, in sdf3d primitive) {
    vec3 q = p - c * clamp(round(p / c), -l, l);
    return primitive(q);
}

// This produces a rectangle of ls.x x l.y instances. Naturally, you can create
// any rectangular shape you want (or other shape) by changing the limits of the
// clamp or the clamp function itself. Also note that the function can be
// specialized to 2 or 1 dimensions easily.

// Deformations and distortions

// Deformations and distortions allow to enhance the shape of primitives or even
// fuse different primitives together. The operations usually distort the
// distance field and make it non euclidean anymore, so one must be careful when
// raymarching them, you will probably need to decrease your step size, if you
// are using a raymarcher to sample this. In principle one can compute the
// factor by which the step size needs to be reduced (inversely proportional to
// the compression of the space, which is given by the Jacobian of the
// deformation function). But even with dual numbers or automatic
// differentiation, it's usually just easier to find the constant by hand for a
// given primitive.

// I'd say that while it is tempting to use a distortion or displacement to
// achieve a given shape, and I often use them myself of course, it is sometimes
// better to get as close to the desired shape with actual exact euclidean
// primitive operations (elongation, rounding, onioning, union) or tight bounded
// functions (intersection, subtraction) and then only apply as small of a
// distortion or displacement as possible. That way the field stays as close as
// possible to an actual distance field, and the raymarcher will be faster.

// Displacement

// The displacement example below is using sin(20*p.x)*sin(20*p.y)*sin(20*p.z)
// as displacement pattern, but you can of course use anything you might
// imagine.

float opDisplace(in sdf3d primitive, in vec3 p) {
    float d1 = primitive(p);
    float d2 = displacement(p);
    return d1 + d2;
}

// Twist

float opTwist(in sdf3d primitive, in vec3 p) {
    const float k = 10.0; // or some other amount
    float c = cos(k * p.y);
    float s = sin(k * p.y);
    mat2 m = mat2(c, -s, s, c);
    vec3 q = vec3(m * p.xz, p.y);
    return primitive(q);
}

// Bend

float opCheapBend(in sdf3d primitive, in vec3 p) {
    const float k = 10.0; // or some other amount
    float c = cos(k * p.x);
    float s = sin(k * p.x);
    mat2 m = mat2(c, -s, s, c);
    vec3 q = vec3(m * p.xy, p.z);
    return primitive(q);
}
