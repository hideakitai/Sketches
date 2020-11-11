precision highp float;
uniform vec2 resolution;
uniform float time;
uniform vec2 mouse;
uniform sampler2D backbuffer;

#define PI 3.141592
#define EPS 0.0001           // epsilon value (to get normal from diff of ray)
#define RAY_HIT_THRESH 0.001 // ray hit detect detection

// roundness of box
#define BOX_ROUNDNESS 0.01

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - BOX_ROUNDNESS;
}

vec3 normalBox(vec3 p, vec3 b, float eps) {
    // normal is gradient of distance function
    return normalize(vec3(
        sdBox(p + vec3(eps, 0., 0.), b) - sdBox(p + vec3(-eps, 0., 0.), b),
        sdBox(p + vec3(0., eps, 0.), b) - sdBox(p + vec3(0., -eps, 0.), b),
        sdBox(p + vec3(0., 0., eps), b) - sdBox(p + vec3(0., 0., -eps), b)));
}

float sdBoxRepeat(vec3 p, vec3 b, float d) {
    return sdBox(mod(p, d) - d * 0.5, b);
}

vec3 normalBoxRepeat(vec3 p, vec3 b, float d, float eps) {
    // normal is gradient of distance function
    return normalize(vec3(
        sdBoxRepeat(p + vec3(eps, 0., 0.), b, d) - sdBoxRepeat(p + vec3(-eps, 0., 0.), b, d),
        sdBoxRepeat(p + vec3(0., eps, 0.), b, d) - sdBoxRepeat(p + vec3(0., -eps, 0.), b, d),
        sdBoxRepeat(p + vec3(0., 0., eps), b, d) - sdBoxRepeat(p + vec3(0., 0., -eps), b, d)));
}

void main() {
    vec2 p = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // camera
    vec3 cam_pos = vec3(0., 0., 3. + time);
    vec3 cam_dir = vec3(0., 0., -1.);
    vec3 cam_up = vec3(0., 1., 0.);
    vec3 cam_side = cross(cam_dir, cam_up);
    float cam_fov = 60. * 0.5 * PI / 180.;
    float target_depth = 1.;

    // light
    vec3 light_dir = vec3(0.3, 0.5, 1.);

    // ray
    vec3 ray = vec3(sin(cam_fov) * p.x, sin(cam_fov) * p.y, -cos(cam_fov));
    vec3 ray_norm = normalize(ray);

    // box
    vec3 box_size = vec3(0.5);
    float box_repeat = 4.;

    // marching loop (sphere tracing)
    const int n_loop = 64;
    float dist = 0.;
    float ray_len = 0.;
    vec3 ray_pos = vec3(0.);
    for (int i = 0; i < n_loop; ++i) {
        dist = sdBoxRepeat(ray_pos, box_size, box_repeat);
        ray_len += dist;
        ray_pos = cam_pos + ray_norm * ray_len;
    }

    // hit check
    if (abs(dist) < RAY_HIT_THRESH) {
        vec3 normal = normalBoxRepeat(ray_pos, box_size, box_repeat, EPS);
        // vec3 normal = normalSphere(ray_pos, sphere_size, EPS);
        float diffuse = clamp(dot(light_dir, normal), 0.1, 1.);
        gl_FragColor = vec4(vec3(diffuse), 1.);
    } else {
        gl_FragColor = vec4(vec3(0.), 1.);
    }
}
