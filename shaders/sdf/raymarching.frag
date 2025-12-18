#version 450

// Fragment shader para SDF Ray Marching
// Implementación pura Vulkan (basado en ADead-Vector3D)

#include "primitives.glsl"

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

// Uniforms
layout(binding = 0) uniform Config {
    mat4 view;
    mat4 proj;
    mat4 invView;
    mat4 invProj;
    vec3 cameraPos;
    float time;
    uint maxSteps;
    float maxDistance;
    float epsilon;
    uint enableAA;
    uint enableShadows;
    uint enableAO;
} config;

// Scene data (hasta 16 primitivas)
layout(binding = 1) uniform SceneData {
    vec4 primitives[16];  // xyz = center/params, w = type
    vec4 colors[16];      // rgb = color, a = unused
    vec4 params[16];      // Parámetros adicionales
    uint primitiveCount;
    uint operationType;   // 0=union, 1=smooth union, etc.
    float smoothness;
} scene;

// Función de escena - combina todas las primitivas
float sceneSDF(vec3 p) {
    if (scene.primitiveCount == 0) {
        // Escena por defecto: cubo simple
        return sdBox(p, vec3(0.0), vec3(1.0));
    }
    
    float dist = 1000000.0;
    
    for (uint i = 0; i < scene.primitiveCount && i < 16; i++) {
        vec4 prim = scene.primitives[i];
        vec4 param = scene.params[i];
        uint type = uint(prim.w);
        
        float d = 0.0;
        
        // Evaluar primitiva según tipo
        if (type == 0) {
            // Sphere
            d = sdSphere(p, prim.xyz, param.x);
        } else if (type == 1) {
            // Box
            d = sdBox(p, prim.xyz, param.xyz);
        } else if (type == 2) {
            // Torus
            d = sdTorus(p, prim.xyz, param.x, param.y);
        } else if (type == 3) {
            // Cylinder
            d = sdCylinder(p, prim.xyz, param.x, param.y);
        } else if (type == 4) {
            // Capsule
            d = sdCapsule(p, prim.xyz, param.xyz, param.w);
        } else if (type == 5) {
            // Cone
            d = sdCone(p, prim.xyz, param.x, param.y);
        }
        
        // Combinar con operación CSG
        if (i == 0) {
            dist = d;
        } else {
            if (scene.operationType == 0) {
                dist = opUnion(dist, d);
            } else if (scene.operationType == 1) {
                dist = opSmoothUnion(dist, d, scene.smoothness);
            } else if (scene.operationType == 2) {
                dist = opSubtract(dist, d);
            } else if (scene.operationType == 3) {
                dist = opIntersect(dist, d);
            }
        }
    }
    
    return dist;
}

// Calcular normal usando gradiente
vec3 calcNormal(vec3 p) {
    const float eps = 0.001;
    vec2 e = vec2(eps, 0.0);
    
    return normalize(vec3(
        sceneSDF(p + e.xyy) - sceneSDF(p - e.xyy),
        sceneSDF(p + e.yxy) - sceneSDF(p - e.yxy),
        sceneSDF(p + e.yyx) - sceneSDF(p - e.yyx)
    ));
}

// Ray marching principal
vec4 rayMarch(vec3 ro, vec3 rd) {
    float t = 0.0;
    vec3 color = vec3(0.0);
    
    for (uint i = 0; i < config.maxSteps; i++) {
        vec3 p = ro + rd * t;
        float d = sceneSDF(p);
        
        if (d < config.epsilon) {
            // Hit! Calcular iluminación
            vec3 normal = calcNormal(p);
            
            // Iluminación simple
            vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
            float diff = max(dot(normal, lightDir), 0.0);
            
            // Color base (gris para cubo)
            color = vec3(0.6) * (0.3 + 0.7 * diff);
            
            // Anti-aliasing usando fwidth (ADead-AA)
            if (config.enableAA != 0) {
                float aa = sdfAntialiasing(d);
                color *= aa;
            }
            
            return vec4(color, 1.0);
        }
        
        t += d;
        
        if (t > config.maxDistance) {
            break;
        }
    }
    
    // Miss - background
    return vec4(0.1, 0.1, 0.15, 1.0);
}

void main() {
    // Convertir UV a ray direction
    vec2 uv = fragUV * 2.0 - 1.0;
    uv.y = -uv.y;  // Flip Y
    
    // Reconstruir ray desde cámara
    vec4 clipPos = vec4(uv, -1.0, 1.0);
    vec4 viewPos = config.invProj * clipPos;
    viewPos /= viewPos.w;
    
    vec3 rayDir = normalize((config.invView * vec4(viewPos.xyz, 0.0)).xyz);
    vec3 rayOrigin = config.cameraPos;
    
    // Ray march
    outColor = rayMarch(rayOrigin, rayDir);
}
