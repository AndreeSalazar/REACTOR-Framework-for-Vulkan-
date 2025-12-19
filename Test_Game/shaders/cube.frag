#version 450

// Input: From vertex shader
layout(location = 0) in vec3 fragColor;

// Output: Final color
layout(location = 0) out vec4 outColor;

void main() {
    // Simple color output
    outColor = vec4(fragColor, 1.0);
}
