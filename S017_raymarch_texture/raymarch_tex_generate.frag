precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592
#define EPS 0.0001           // epsilon value (to get normal from diff of ray)
#define RAY_HIT_THRESH 0.001 // ray hit detect detection

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

float sdTorus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xz) - t.x, p.y);
    // vec2 q = vec2(length(p.xy) - t.x, p.z);
    return length(q) - t.y;
}

float sdFloor(vec3 p) {
    return dot(p, vec3(0., 1., 0.)) + 2.5;
}

float sdScene(vec3 p, vec2 t) {
    vec3 q = rotate3d(time, vec3(1., 0.25, 0.5)) * p;
    float sd_torus = sdTorus(q, t);
    float sd_floor = sdFloor(p);
    return min(sd_torus, sd_floor);
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
    vec3 cam_pos = vec3(0., 5., 5.);
    float cam_fov = 90. * 0.5 * PI / 180.;
    float target_depth = 1.;

    // light
    vec3 light_dir = vec3(-0.5, 0.5, 0.5);

    // ray
    vec3 ray = vec3(sin(cam_fov) * p.x, sin(cam_fov) * p.y, -cos(cam_fov));
    ray = lookAt(cam_pos, vec3(0.), 0.) * ray;
    vec3 ray_norm = normalize(ray);

    // torus
    float torus_outer_radius = 2.;
    float torus_inner_radius = 0.5;
    vec2 torus_radius = vec2(torus_outer_radius, torus_inner_radius);

    // marching loop (sphere tracing)
    const int n_loop = 256;
    float dist = 0.;
    float ray_len = 0.;
    vec3 ray_pos = vec3(0.);
    for (int i = 0; i < n_loop; ++i) {
        dist = sdScene(ray_pos, torus_radius);
        if (dist < RAY_HIT_THRESH) // stop ray if hit
            break;
        ray_len += dist;
        ray_pos = cam_pos + ray_norm * ray_len;
    }

    // hit check
    if (abs(dist) < RAY_HIT_THRESH) {
        vec3 normal = getNormal(ray_pos, torus_radius, EPS);
        float diffuse = clamp(dot(light_dir, normal), 0.1, 1.);

        // generate tile pattern on the surface
        float u = 1. - floor(mod(ray_pos.x, 2.));
        float v = 1. - floor(mod(ray_pos.z, 2.));
        if ((u == 1. && v < 1.) || (u < 1. && v == 1.))
            diffuse *= 0.7;

        gl_FragColor = vec4(vec3(diffuse), 1.);
    } else {
        gl_FragColor = vec4(vec3(0.), 1.);
    }
}
