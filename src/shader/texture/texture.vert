#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec2 a_tex_coords;
layout(set = 1, binding = 0) uniform MatrixUniform {
    mat4 view;
};
layout(set = 1, binding = 1) uniform MatrixUniform {
    mat4 proj;
};

layout(location = 0) out vec2 v_tex_coords;


const mat4 OPENGL_TO_WGPU_MATRIX = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0
);

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = OPENGL_TO_WGPU_MATRIX * proj * view * vec4(a_position, 1.0);
}