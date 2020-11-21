#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec2 a_tex_coords;
layout(location = 2) in vec3 a_normal;

// all coordinates are view space
layout(location = 0) out vec2 v_tex_coords;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec3 v_position;
layout(location = 3) out vec3 v_light_position;
layout(location = 4) out vec3 v_eye_position;

layout(set = 1, binding = 0) uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_matrix;
    mat4 u_proj_matrix;
};

layout(set = 2, binding = 0) uniform Light {
    vec3 u_light_position;
    vec3 u_light_color;
};

layout(set = 3, binding = 0) buffer Instances {
    mat4 s_models[];
};

void main() {
    v_tex_coords = a_tex_coords;

    // calcurate all position in view space
    mat4 model_matrix = s_models[gl_InstanceIndex];
    mat3 normal_matrix = mat3(transpose(inverse(u_view_matrix * model_matrix)));
    v_normal = normal_matrix * a_normal;
    vec4 view_space = u_view_matrix * model_matrix * vec4(a_position, 1.0);
    v_position = view_space.xyz;
    v_light_position = (u_view_matrix * vec4(u_light_position, 1.)).xyz;
    v_eye_position = (u_view_matrix * vec4(u_view_position, 1.)).xyz;

    gl_Position = u_proj_matrix * view_space;
}
