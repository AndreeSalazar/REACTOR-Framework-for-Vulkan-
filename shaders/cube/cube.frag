#version 450

// Fragment shader con Phong shading profesional
layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    // Normalize interpolated normal
    vec3 normal = normalize(fragNormal);
    
    // Light properties
    vec3 lightPos = vec3(5.0, 5.0, 5.0);
    vec3 lightColor = vec3(1.0, 1.0, 1.0);
    vec3 viewPos = vec3(3.0, 3.0, 3.0);
    
    // Ambient
    float ambientStrength = 0.3;
    vec3 ambient = ambientStrength * lightColor;
    
    // Diffuse
    vec3 lightDir = normalize(lightPos - fragWorldPos);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // Specular (Phong)
    float specularStrength = 0.6;
    vec3 viewDir = normalize(viewPos - fragWorldPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = specularStrength * spec * lightColor;
    
    // Combine
    vec3 result = (ambient + diffuse + specular) * fragColor;
    
    outColor = vec4(result, 1.0);
}
