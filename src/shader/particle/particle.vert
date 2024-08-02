#version 450

layout(set = 1, binding = 0) uniform MatrixUniform {
    mat4 view;
};
layout(set = 1, binding = 1) uniform MatrixUniform {
    mat4 proj;
};

struct ParticleLl {
    vec3 position;
};

struct DensityLl {
    vec2 density;
};

layout(set = 0, binding = 0) buffer ParticleBuffer { ParticleLl particles[]; };
layout(set = 0, binding = 2) buffer DensityBuffer { DensityLl densities[]; };
layout(set = 2, binding = 0) uniform RadiusLl {
    float radius;
};


layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) out vec2 particlePosition;


const vec3 quadVertices[4] = vec3[](
    vec3(-1.0, -1.0, 0.0),
    vec3( 1.0, -1.0, 0.0),
    vec3(-1.0,  1.0, 0.0),
    vec3( 1.0,  1.0, 0.0)
);

const vec2 texCoords[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0)
);

const mat4 OPENGL_TO_WGPU_MATRIX = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0
);

vec3 gradient(float t) {
    vec3 color;

    if (t < 0.25) {
        float f = t / 0.25;
        color = mix(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 1.0), f); // Blue to Cyan
    } else if (t < 0.5) {
        float f = (t - 0.25) / 0.25;
        color = mix(vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 0.0), f); // Cyan to Green
    } else if (t < 0.75) {
        float f = (t - 0.5) / 0.25;
        color = mix(vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0), f); // Green to Yellow
    } else {
        float f = (t - 0.75) / 0.25;
        color = mix(vec3(1.0, 1.0, 0.0), vec3(1.0, 0.5, 0.0), f); // Yellow to Orange
    }

    return color;
}

float get_max_density() {
    uint numParticles = densities.length();
    float maxDens = 0.0;

    for (uint i = 0; i < numParticles; ++i) {
        if (densities[i].density.x > maxDens) {
            maxDens = densities[i].density.x;
        }
    }
    return maxDens;
}
void main() {
    uint particleIndex = gl_InstanceIndex;
    uint vertexId = gl_VertexIndex % 4;

    vec3 offset = quadVertices[vertexId] * radius; //particles[particleIndex].radius;
    vec4 worldPosition = vec4(particles[particleIndex].position + offset, 1.0);

    gl_Position = OPENGL_TO_WGPU_MATRIX * proj * view * worldPosition;

    fragTexCoord = texCoords[vertexId];
    particlePosition = particles[particleIndex].position.xy;

    float maxDensity = get_max_density();
    fragColor = gradient(densities[particleIndex].density.x / maxDensity);
}

