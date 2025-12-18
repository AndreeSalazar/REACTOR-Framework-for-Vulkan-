#version 450

// Fragment shader con modos de debug
layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

// Push constant para modo de visualización
layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    int debugMode;  // 0=Normal, 1=Wireframe, 2=Normals, 3=Depth, etc.
    float padding[3];
} push;

void main() {
    vec3 normal = normalize(fragNormal);
    
    // Modo 0: Normal (Phong Shading)
    if (push.debugMode == 0) {
        // Light properties
        vec3 lightPos = vec3(8.0, 8.0, 8.0);
        vec3 lightColor = vec3(1.2, 1.2, 1.2); // Más brillante
        
        // Ambient más fuerte para mejor claridad
        float ambientStrength = 0.4;
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
    // Modo 1: Wireframe - Solo bordes brillantes
    else if (push.debugMode == 1) {
        // Detectar bordes usando derivadas
        vec3 dx = dFdx(fragWorldPos);
        vec3 dy = dFdy(fragWorldPos);
        float edge = length(dx) + length(dy);
        
        // Bordes brillantes en cyan sobre fondo negro
        if (edge > 0.05) {
            outColor = vec4(0.0, 1.0, 1.0, 1.0); // Cyan brillante
        } else {
            outColor = vec4(0.0, 0.0, 0.0, 1.0); // Negro
        }
    }
    // Modo 2: Normals (RGB)
    else if (push.debugMode == 2) {
        // Convertir normales [-1,1] a colores [0,1]
        vec3 normalColor = normal * 0.5 + 0.5;
        outColor = vec4(normalColor, 1.0);
    }
    // Modo 3: Depth Buffer
    else if (push.debugMode == 3) {
        // Visualizar profundidad (cerca=blanco, lejos=negro)
        float depth = gl_FragCoord.z;
        // Linearizar depth para mejor visualización
        float near = 0.1;
        float far = 100.0;
        float linearDepth = (2.0 * near) / (far + near - depth * (far - near));
        outColor = vec4(vec3(linearDepth), 1.0);
    }
    // Modo 4: Importance Map (ISR)
    else if (push.debugMode == 4) {
        // Simular mapa de importancia basado en gradientes
        vec3 dx = dFdx(fragColor);
        vec3 dy = dFdy(fragColor);
        float gradient = length(dx) + length(dy);
        
        // Mapa de calor: azul (baja) -> verde -> amarillo -> rojo (alta)
        vec3 heatColor;
        if (gradient < 0.1) {
            heatColor = vec3(0.0, 0.0, 1.0); // Azul
        } else if (gradient < 0.3) {
            heatColor = mix(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), (gradient - 0.1) / 0.2);
        } else if (gradient < 0.6) {
            heatColor = mix(vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0), (gradient - 0.3) / 0.3);
        } else {
            heatColor = mix(vec3(1.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), (gradient - 0.6) / 0.4);
        }
        
        outColor = vec4(heatColor, 1.0);
    }
    // Modo 5: Pixel Sizing (ISR)
    else if (push.debugMode == 5) {
        // Visualizar tamaño de píxeles adaptativos
        // Basado en importancia (gradientes)
        vec3 dx = dFdx(fragColor);
        vec3 dy = dFdy(fragColor);
        float gradient = length(dx) + length(dy);
        
        // Píxeles grandes (bajo detalle) = verde
        // Píxeles pequeños (alto detalle) = rojo
        float pixelSize = 1.0 - gradient;
        vec3 sizeColor = mix(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), pixelSize);
        
        outColor = vec4(sizeColor, 1.0);
    }
    // Modo 6: Temporal Coherence (ISR)
    else if (push.debugMode == 6) {
        // Visualizar coherencia temporal (simulado)
        // Mostrar áreas estables vs cambiantes
        vec3 dx = dFdx(fragWorldPos);
        vec3 dy = dFdy(fragWorldPos);
        float motion = length(dx) + length(dy);
        
        // Estable = azul, Cambiante = rojo
        vec3 temporalColor = mix(vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), motion * 10.0);
        
        outColor = vec4(temporalColor, 1.0);
    }
    else {
        // Fallback: color base
        outColor = vec4(fragColor, 1.0);
    }
}
