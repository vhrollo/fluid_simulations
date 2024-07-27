layout(set = 0, binding = 0) uniform MatrixUniform {
    mat4 view;
};
layout(set = 0, binding = 1) uniform MatrixUniform {
    mat4 proj;
};

layout(set = 1, binding = 0) uniform float radius;

const vec3 quadVertices[4] = vec3[](
    vec3(-1.0, -1.0, 0.0),
    vec3( 1.0, -1.0, 0.0),
    vec3(-1.0,  1.0, 0.0),
    vec3( 1.0,  1.0, 0.0)
);

const mat4 OPENGL_TO_WGPU_MATRIX = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0
);

const vec2 texCoords[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0)
);

layout(location = 0) out vec2 fragTexCoord;

void main() {
    uint vertexId = gl_VertexIndex % 4;
    vec3 position = quadVertices[vertexId] * radius;
    gl_Position = OPENGL_TO_WGPU_MATRIX * proj * view * vec4(position, 1.0);

    fragTexCoord = texCoords[vertexId];
}