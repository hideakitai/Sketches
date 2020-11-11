precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592
#define EPS 0.0001           // epsilon value (to get normal from diff of ray)
#define RAY_SCALE 1.0        // ray march step scaling
#define RAY_HIT_THRESH 0.001 // ray hit detect detection

float smoothmin(float a, float b, float k) {
    float h = exp(-k * a) + exp(-k * b);
    return -log(h) / k;
}

mat3 rotate3d(float rad, vec3 axis) {
    vec3 a = normalize(axis);
    float s = sin(rad);
    float c = cos(rad);
    float r = 1.0 - c;
    return mat3(
        a.x * a.x * r + c,
        a.y * a.x * r + a.z * s,
        a.z * a.x * r - a.y * s,
        a.x * a.y * r - a.z * s,
        a.y * a.y * r + c,
        a.z * a.y * r + a.x * s,
        a.x * a.z * r + a.y * s,
        a.y * a.z * r - a.x * s,
        a.z * a.z * r + c);
}

vec3 twistX(vec3 p, float power) {
    float s = sin(power * p.x);
    float c = cos(power * p.x);
    mat3 m = mat3(
        1.0, 0.0, 0.0,
        0.0, c, s,
        0.0, -s, c);
    return m * p;
}

vec3 twistY(vec3 p, float power) {
    float s = sin(power * p.y);
    float c = cos(power * p.y);
    mat3 m = mat3(
        c, 0.0, -s,
        0.0, 1.0, 0.0,
        s, 0.0, c);
    return m * p;
}

vec3 twistZ(vec3 p, float power) {
    float s = sin(power * p.z);
    float c = cos(power * p.z);
    mat3 m = mat3(
        c, s, 0.0,
        -s, c, 0.0,
        0.0, 0.0, 1.0);
    return m * p;
}

float sdTorus(vec3 p, vec2 t) {
    // vec2 q = vec2(length(p.xz) - t.x, p.y);
    vec2 q = vec2(length(p.xy) - t.x, p.z);
    return length(q) - t.y;
}

float sdFloor(vec3 p) {
    return dot(p, vec3(0., 1., 0.)) + 1.0;
}

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdScene(vec3 p, vec2 t) {
    // p = rotate3d(time, vec3(1., 0., 1.)) * p;
    // p = twistX(p, sin(time));
    p = twistY(p, cos(time));
    // p = twistZ(p, sin(time));
    float sd_torus = sdTorus(p, t);
    float sd_box = sdBox(p, vec3(2., 0.1, 0.5)) - 0.1;
    // return min(sd_torus, sd_box);            // union (OR)
    return smoothmin(sd_torus, sd_box, 10.); // smooth (OR)
    // return max(sd_torus, sd_box);            // boolean (AND)
    // return max(-sd_torus, sd_box);           // boolean (XOR)
    // return max(sd_torus, -sd_box);           // boolean (XOR)
}

vec3 getNormal(vec3 p, vec2 t, float eps) {
    // normal is gradient of distance function
    return normalize(vec3(
        sdScene(p + vec3(eps, 0., 0.), t) - sdScene(p + vec3(-eps, 0., 0.), t),
        sdScene(p + vec3(0., eps, 0.), t) - sdScene(p + vec3(0., -eps, 0.), t),
        sdScene(p + vec3(0., 0., eps), t) - sdScene(p + vec3(0., 0., -eps), t)));
}

// https://github.com/glslify/glsl-look-at
mat3 lookAt(vec3 origin, vec3 target, float roll) {
    vec3 rr = vec3(sin(roll), cos(roll), 0.0);
    vec3 ww = normalize(target - origin);
    vec3 uu = normalize(cross(ww, rr));
    vec3 vv = normalize(cross(uu, ww));
    return mat3(uu, vv, -ww);
}

void main() {
    vec2 p = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // camera
    vec3 cam_pos = vec3(0., 1.5, 3.);
    vec3 cam_dir = vec3(0.577, -0.577, -0.577);
    vec3 cam_up = vec3(0.577, 0.577, -0.577);
    vec3 cam_side = cross(cam_dir, cam_up);
    float cam_fov = 90. * 0.5 * PI / 180.;
    float target_depth = 1.;

    // light
    vec3 light_dir = vec3(-0.577, 0.577, 0.577);

    // ray
    vec3 ray = vec3(sin(cam_fov) * p.x, sin(cam_fov) * p.y, -cos(cam_fov));
    ray = lookAt(cam_pos, vec3(0.), 0.) * ray;
    vec3 ray_norm = normalize(ray);

    // torus
    float torus_outer_radius = 1.5;
    float torus_inner_radius = 0.25;
    vec2 torus_radius = vec2(torus_outer_radius, torus_inner_radius);

    // marching loop (sphere tracing)
    const int n_loop = 256;
    float dist = 0.;
    float ray_len = 0.;
    vec3 ray_pos = vec3(0.);
    for (int i = 0; i < n_loop; ++i) {
        dist = sdScene(ray_pos, torus_radius);
        ray_len += dist;
        ray_pos = cam_pos + ray_norm * ray_len * RAY_SCALE;
    }

    // hit check
    if (abs(dist) < RAY_HIT_THRESH) {
        vec3 normal = getNormal(ray_pos, torus_radius, EPS);
        float diffuse = clamp(dot(light_dir, normal), 0.1, 1.);
        gl_FragColor = vec4(vec3(diffuse), 1.);
    } else {
        gl_FragColor = vec4(vec3(0.), 1.);
    }
}
