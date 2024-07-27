#version 450

layout(set = 0, binding = 0) uniform MatrixUniform {
    mat4 view;
};
layout(set = 0, binding = 1) uniform MatrixUniform {
    mat4 proj;
};

layout(set = 1, binding = 0) buffer readonly BoundingBoxLl{
    vec2 box_position;
    vec2 size;
};


const mat4 OPENGL_TO_WGPU_MATRIX = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0
);

layout(location = 0) in vec2 position;



void main() {
    vec2 new_position = position * size / 2.0 + box_position;
    gl_Position = OPENGL_TO_WGPU_MATRIX * proj * view *vec4(new_position, 0.0, 1.0);
}