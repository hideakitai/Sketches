precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592
#define EPS 0.0001           // epsilon value (to get normal from diff of ray)
#define RAY_HIT_THRESH 0.001 // ray hit detect detection

float sdTorus(vec3 p, vec2 t) {
    // vec2 q = vec2(length(p.xz) - t.x, p.y);
    vec2 q = vec2(length(p.xy) - t.x, p.z);
    return length(q) - t.y;
}

vec3 normalTorus(vec3 p, vec2 t, float eps) {
    // normal is gradient of distance function
    return normalize(vec3(
        sdTorus(p + vec3(eps, 0., 0.), t) - sdTorus(p + vec3(-eps, 0., 0.), t),
        sdTorus(p + vec3(0., eps, 0.), t) - sdTorus(p + vec3(0., -eps, 0.), t),
        sdTorus(p + vec3(0., 0., eps), t) - sdTorus(p + vec3(0., 0., -eps), t)));
}

void main() {
    vec2 p = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // camera
    vec3 cam_pos = vec3(0., 0., 3.);
    vec3 cam_dir = vec3(0., 0., -1.);
    vec3 cam_up = vec3(0., 1., 0.);
    vec3 cam_side = cross(cam_dir, cam_up);
    float cam_fov = 90. * 0.5 * PI / 180.;
    float target_depth = 1.;

    // light
    vec3 light_dir = vec3(-0.5, 0.5, 0.5);

    // ray
    vec3 ray = vec3(sin(cam_fov) * p.x, sin(cam_fov) * p.y, -cos(cam_fov));
    vec3 ray_norm = normalize(ray);

    // torus
    float torus_outer_radius = 0.75;
    float torus_inner_radius = 0.25;
    vec2 torus_radius = vec2(torus_outer_radius, torus_inner_radius);

    // marching loop (sphere tracing)
    const int n_loop = 16;
    float dist = 0.;
    float ray_len = 0.;
    vec3 ray_pos = vec3(0.);
    for (int i = 0; i < n_loop; ++i) {
        dist = sdTorus(ray_pos, torus_radius);
        ray_len += dist;
        ray_pos = cam_pos + ray_norm * ray_len;
    }

    // hit check
    if (abs(dist) < RAY_HIT_THRESH) {
        vec3 normal = normalTorus(ray_pos, torus_radius, EPS);
        float diffuse = clamp(dot(light_dir, normal), 0.1, 1.);
        gl_FragColor = vec4(vec3(diffuse), 1.);
    } else {
        gl_FragColor = vec4(vec3(0.), 1.);
    }
}
