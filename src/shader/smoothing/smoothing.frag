layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

void main() {
    vec2 center = vec2(0.5, 0.5);
    float radius = 0.5;
    float innerRadius = 0.49;
    vec2 circCoord = fragTexCoord - center;
    
    if (dot(circCoord, circCoord) > radius * radius || innerRadius * innerRadius > dot(circCoord, circCoord)) {
        discard;
    }

    outColor = vec4(1.0);
}