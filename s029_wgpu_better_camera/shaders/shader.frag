#version 450

#define USE_NORMAL_MAP

// all coordinates are view space
#ifdef USE_NORMAL_MAP
layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_eye_position;
layout(location = 2) in vec3 v_light_position;
layout(location = 3) in vec2 v_tex_coords;
layout(location = 4) in mat3 v_tangent_matrix;
#else
layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_eye_position;
layout(location = 2) in vec3 v_light_position;
layout(location = 3) in vec2 v_tex_coords;
layout(location = 4) in vec3 v_normal;
#endif

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0)
    uniform Uniforms {
    vec3 u_view_position; // unused
    mat4 u_view_matrix;
    mat4 u_proj_matrix;
};

layout(set = 2, binding = 0) uniform Light {
    vec3 u_light_position;
    vec3 u_light_color;
};

layout(set = 3, binding = 0) uniform texture2D t_diffuse;
layout(set = 3, binding = 1) uniform sampler s_diffuse;
layout(set = 3, binding = 2) uniform texture2D t_normal;
layout(set = 3, binding = 3) uniform sampler s_normal;

void main() {
    vec4 object_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    vec4 object_normal = texture(sampler2D(t_normal, s_normal), v_tex_coords); // NEW!

    // We don't need (or want) much ambient light, so 0.1 is fine
    float ambient_strength = 0.1;
    vec3 ambient_color = u_light_color * ambient_strength;

#ifdef USE_NORMAL_MAP
    vec3 normal = normalize(object_normal.rgb);
#else
    vec3 normal = normalize(v_normal);
#endif
    vec3 light_dir = normalize(v_light_position - v_position);
    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    vec3 diffuse_color = u_light_color * diffuse_strength;

    vec3 view_dir = normalize(v_eye_position - v_position);
    vec3 half_dir = normalize(view_dir + light_dir);
    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32);
    vec3 specular_color = specular_strength * u_light_color;

    vec3 result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    // Since lights don't typically (afaik) cast transparency, so we use
    // the alpha here at the end.
    f_color = vec4(result, object_color.a);
}
