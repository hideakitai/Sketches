#define PI 3.141592653589793
#define TAU 6.283185307179586

#ifndef saturate
#define saturate(x) clamp(x, 0., 1.)
#endif

float sumabs(vec2 p) { return abs(p.x) + abs(p.y); }
float maxabs(vec2 p) { return max(abs(p.x), abs(p.y)); }

float dot2(in vec2 v) { return dot(v, v); }
float dot2(in vec3 v) { return dot(v, v); }

float ndot(in vec2 a, in vec2 b) { return a.x * b.x - a.y * b.y; }

// float cross(vec2 v1, vec2 v2) { return v1.x * v2.y - v1.y * v2.x; }
float cross(vec2 v1, vec2 v2) { return dot(v1, vec2(v2.y, -v2.x)); }

vec3 pow(vec3 c, float p) { return vec3(pow(c.r, p), pow(c.g, p), pow(c.b, p)); }
