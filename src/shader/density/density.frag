layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 outColor;

float gaussian(float distance) {
    float radius = 0.5;
    float sigma = radius / 3.0;  // Adjust sigma to control the width of the curve
    return exp(-0.5 * pow(distance / sigma, 2.0));
}

void main() {
    vec2 center = vec2(0.5, 0.5);
    float radius = 0.5;
    vec2 circCoord = fragTexCoord - center;
    float distance = length(circCoord);
    
    if (distance > radius) {
        discard;
    }

    float a = gaussian(distance);

    vec3 darkBlue = vec3(0.0, 0.0, 0.5);
    vec3 lightBlue = vec3(0.5, 0.5, 1.0);

    // Adjust the mix factor to weight the light blue less
    float adjustedMixFactor = pow(a, 1.5); // Using sqrt to reduce the weight of the light blue
    vec3 finalColor = mix(darkBlue, lightBlue, adjustedMixFactor);

    outColor = vec4(finalColor, a);
}
