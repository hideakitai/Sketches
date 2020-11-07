//  Function from Iñigo Quiles
//  www.iquilezles.org/www/articles/functions/functions.htm

#define PI 3.14159265359
#define TWO_PI 6.28318530718

// Almost Identity (I)
// Imagine you don't want to change a value unless it's zero or very close to it,
// in which case you want to replace the value with a small constant.
// Then, rather than doing a conditional branch which introduces a discontinuity,
// you can smoothly blend your value with your threshold.
// Let m be the threshold (anything above m stays unchanged),
// and n the value things will take when your input is zero. Then,
// the following function does the soft clipping (in a cubic fashion):
float almost_identity(float x, float low_clip, float interp_end) {
    if (x > interp_end)
        return x;
    float a = 2.0 * low_clip - interp_end;
    float b = 2.0 * interp_end - 3.0 * low_clip;
    float t = x / interp_end;
    return (a * t + b) * t * t + low_clip;
}

// Almost Unit Identity
// This is another near-identiy function, but this one maps the unit interval to itself.
// But it is special in that not only remaps 0 to 0 and 1 to 1,
// but has a 0 derivative at the origin and a derivative of 1 at 1,
// making it ideal for transitioning things from being stationary to being
// in motion as if they had been in motion the whole time.
// It's equivalent to the Almost Identiy above with n=0 and m=1, basically.
// And since it's a cubic just like smoothstep() and therefore very fast to evaluate:
float almost_unit_identity(float x) {
    return x * x * (2.0 - x);
}

// Almost Identity (II)
// A different way to achieve a near identity that can also be used as smooth-abs() is
// through the square root of a biased square, instead of a cubic polynomail.
// I saw this technique first in a shader by user "omeometo" in Shadertoy.
// This approach can be a bit slower than the cubic above, depending on the hardware.
// And while it has zero derivative, it has a non-zero second derivative,
// which could cause problems in some situations:
float almost_identity(float x, float n) {
    return sqrt(x * x + n);
}

// Exponential Impulse
// Great for triggering behaviours or making envelopes for music or animation,
// and for anything that grows fast and then slowly decays.
// Use k to control the stretching of the function.
// Btw, its maximum, which is 1, happens at exactly x = 1/k.
float exp_impulse(float x, float k) {
    float h = k * x;
    return h * exp(1.0 - h);
}

// Sustained Impulse
// Similar to the previous, but it allows for control on the width of attack
// (through the parameter "k") and the release (parameter "f") independently.
// Also, it ensures the impulse releases at a value of 1.0 instead of 0.
float exp_sustained_impulse(float x, float k, float f) {
    float s = max(x - f, 0.0);
    return min(x * x / (f * f), 1.0 + (2.0 / f) * s * exp(-k * s));
}

// Polynomial Impulse
// Another impulse function that doesn't use exponentials can be designed by using polynomicals.
// Use k to control falloff of the function.
// For example, a quadratic can be used, which peaks at x = sqrt(1/k).
float quad_impulse(float x, float k) {
    return 2.0 * sqrt(k) * x / (1.0 + k * x * x);
}

// You can easily generalize it to other powers to get different falloff shapes,
// where n is the degree of the polynomial:
// These generalized impulses peak at x = [k(n-1)]-1/n.
float poly_impulse(float k, float n, float x) {
    return (n / (n - 1.0)) * pow((n - 1.0) * k, 1.0 / n) * x / (1.0 + k * pow(x, n));
}

// Cubic Pulse
// Of course you found yourself doing smoothstep(c-w,c,x)-smoothstep(c,c+w,x) very often,
// probably because you were trying to isolate some features in a signal.
// Then, this cubicPulse() will be your new best friend.
// You can also use it as a cheap replacement for a gaussian.
float cubic_pulse(float c, float w, float x) {
    x = abs(x - c);
    if (x > w)
        return 0.0;
    x /= w;
    return 1.0 - x * x * (3.0 - 2.0 * x);
}

// Exponential Step
// A natural attenuation is an exponential of a linearly decaying quantity: yellow curve, exp(-x).
// A gaussian, is an exponential of a quadratically decaying quantity: light green curve, exp(-x2).
// You can generalize and keep increasing powers, and get a sharper and sharper s-shaped curves,
// until you get a step() in the limit.
float exp_step(float x, float k, float n) {
    return exp(-k * pow(x, n));
}

// Gain
// Remapping the unit interval into the unit interval by expanding the sides and compressing the center,
// and keeping 1/2 mapped to 1/2, that can be done with the gain() function.
// This was a common function in RSL tutorials (the Renderman Shading Language).
// k=1 is the identity curve, k<1 produces the classic gain() shape, and k>1 produces "s" shaped curces.
// The curves are symmetric (and inverse) for k=a and k=1/a.
// k<1 on the left, k>1 on the right
float gain(float x, float k) {
    float a = 0.5 * pow(2.0 * ((x < 0.5) ? x : 1.0 - x), k);
    return (x < 0.5) ? a : 1.0 - a;
}

// Parabola
// A nice choice to remap the 0..1 interval into 0..1,
// such that the corners are mapped to 0 and the center to 1.
// In other words, parabola(0) = parabola(1) = 0, and parabola(1/2) = 1.
float parabola(float x, float k) {
    return pow(4.0 * x * (1.0 - x), k);
}

// Power curve
// This is a generalziation of the Parabola() above.
// It also maps the 0..1 interval into 0..1 by keeping the corners mapped to 0.
// But in this generalziation you can control the shape one either side of the curve,
// which comes handy when creating leaves, eyes, and many other interesting shapes.
// Note that k is chosen such that pcurve() reaches exactly 1 at its maximum for illustration purposes,
// but in many applications the curve needs to be scaled anyways
// so the slow computation of k can be simply avoided.
float pcurve(float x, float a, float b) {
    float k = pow(a + b, a + b) / (pow(a, a) * pow(b, b));
    return k * pow(x, a) * pow(1.0 - x, b);
}

// Sinc curve
// A phase shifted sinc curve can be useful if it starts at zero and ends at zero,
// for some bouncing behaviors (suggested by Hubert-Jan).
// Give k different integer values to tweak the amount of bounces.
// It peaks at 1.0, but that take negative values, which can make it unusable in some applications.
float sinc(float x, float k) {
    float a = PI * (k * x - 1.0);
    return sin(a) / a;
}

float plot(vec2 st, float pct, float width) {
    return smoothstep(pct - width / 2., pct, st.y) - smoothstep(pct, pct + width / 2., st.y);
}

float plot(vec2 st, float pct) {
    return plot(st, pct, 0.01);
}

#pragma glslify : export(plot)
