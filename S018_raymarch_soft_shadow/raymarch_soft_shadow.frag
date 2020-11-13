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
    // return mat3(uu, vv, -ww);
    return mat3(uu, vv, ww);
}

float genShadow(vec3 ro, vec3 rd, vec2 tr) {
    float c = 0.001;
    float r = 1.;
    float shadow_coef = 0.5;
    float shadow_diffuse_coef = 16.;
    for (float t = 0.; t < 50.; ++t) {
        float h = sdScene(ro + rd * c, tr);
        if (h < RAY_HIT_THRESH)
            return shadow_coef;
        // diffuse shadow depending on the distance
        r = min(r, h * shadow_diffuse_coef / c);
        c += h;
    }
    return 1. - shadow_coef + r * shadow_coef;
}

void main() {
    vec2 p = (gl_FragCoord.xy * 2. - resolution.xy) / min(resolution.x, resolution.y);

    // camera
    vec3 cam_pos = vec3(0., 5., 5.);
    float cam_fov = 90. * 0.5 * PI / 180.;
    float target_depth = 1.;

    // light
    vec3 light_pos = vec3(2., 2., 2.);
    vec3 light_dir = lookAt(light_pos, vec3(0.), 0.) * vec3(0., 0., -1);
    light_dir = normalize(light_dir);

    // ray
    vec3 ray = vec3(sin(cam_fov) * p.x, sin(cam_fov) * p.y, cos(cam_fov));
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
    vec3 color;
    float shadow = 1.;
    if (abs(dist) < RAY_HIT_THRESH) {
        vec3 normal = getNormal(ray_pos, torus_radius, EPS);

        // light
        float diffuse = clamp(dot(light_dir, normal), 0.1, 1.);
        vec3 half_le = normalize(light_dir - ray);
        float specular = pow(clamp(dot(half_le, normal), 0., 1.), 50.);

        // generate shadow
        // if ray hit the surface, trace ray again to the light
        // if second ray hits any object, generate shadow
        // to avoid that ray hit the object's inner surface,
        // make offset (if ray has passed into the surface a bit, it occurs)
        vec3 ray_offset = normal * 0.001;
        shadow = genShadow(ray_pos + ray_offset, light_dir, torus_radius);

        // generate tile pattern on the surface
        float u = 1. - floor(mod(ray_pos.x, 2.));
        float v = 1. - floor(mod(ray_pos.z, 2.));
        if ((u == 1. && v < 1.) || (u < 1. && v == 1.))
            diffuse *= 0.7;

        color = vec3(diffuse) + vec3(specular);
    } else {
        color = vec3(0.);
    }

    gl_FragColor = vec4(color * max(0.5, shadow), 1.0);
}
