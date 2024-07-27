#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec2 particlePosition;

layout(location = 0) out vec4 outColor;

void main() {
    vec2 center = vec2(0.5, 0.5);
    float radius = 0.5;
    vec2 circCoord = fragTexCoord - center;
    
    if (dot(circCoord, circCoord) > radius * radius) {
        discard;
    }

    // outColor = vec4(fragColor, 1.0);
    outColor = vec4(0.0, 0.0, 0.0, 1.0);
}