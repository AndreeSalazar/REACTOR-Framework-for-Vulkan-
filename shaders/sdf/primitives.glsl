// SDF Primitives Library
// Basado en ADead-Vector3D
// Funciones matemáticas para renderizado SDF

#ifndef SDF_PRIMITIVES_GLSL
#define SDF_PRIMITIVES_GLSL

// ============================================================================
// Primitivas Básicas
// ============================================================================

// Sphere SDF
float sdSphere(vec3 p, vec3 center, float radius) {
    return length(p - center) - radius;
}

// Box SDF
float sdBox(vec3 p, vec3 center, vec3 size) {
    vec3 q = abs(p - center) - size;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Torus SDF
float sdTorus(vec3 p, vec3 center, float majorRadius, float minorRadius) {
    vec3 q = p - center;
    vec2 t = vec2(length(q.xz) - majorRadius, q.y);
    return length(t) - minorRadius;
}

// Cylinder SDF
float sdCylinder(vec3 p, vec3 center, float radius, float height) {
    vec3 q = p - center;
    vec2 d = abs(vec2(length(q.xz), q.y)) - vec2(radius, height);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Capsule SDF
float sdCapsule(vec3 p, vec3 a, vec3 b, float radius) {
    vec3 pa = p - a;
    vec3 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - radius;
}

// Cone SDF
float sdCone(vec3 p, vec3 center, float angle, float height) {
    vec3 q = p - center;
    vec2 c = vec2(sin(angle), cos(angle));
    float d1 = length(q.xz);
    float d2 = -q.y - height;
    float d3 = d1 * c.y - q.y * c.x;
    return max(max(d2, d3), -q.y);
}

// ============================================================================
// CSG Operations
// ============================================================================

// Union (A ∪ B)
float opUnion(float d1, float d2) {
    return min(d1, d2);
}

// Subtraction (A - B)
float opSubtract(float d1, float d2) {
    return max(d1, -d2);
}

// Intersection (A ∩ B)
float opIntersect(float d1, float d2) {
    return max(d1, d2);
}

// Smooth Union
float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

// Smooth Subtraction
float opSmoothSubtract(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

// Smooth Intersection
float opSmoothIntersect(float d1, float d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}

// ============================================================================
// Transformaciones
// ============================================================================

// Repetición infinita
vec3 opRepeat(vec3 p, vec3 spacing) {
    return mod(p + 0.5 * spacing, spacing) - 0.5 * spacing;
}

// Twist
vec3 opTwist(vec3 p, float amount) {
    float c = cos(amount * p.y);
    float s = sin(amount * p.y);
    mat2 m = mat2(c, -s, s, c);
    return vec3(m * p.xz, p.y);
}

// Bend
vec3 opBend(vec3 p, float amount) {
    float c = cos(amount * p.y);
    float s = sin(amount * p.y);
    mat2 m = mat2(c, -s, s, c);
    return vec3(m * p.xy, p.z);
}

// ============================================================================
// Utilidades
// ============================================================================

// Calcular normal usando gradiente
vec3 calcNormal(vec3 p, float dist) {
    const float eps = 0.001;
    vec2 e = vec2(eps, 0.0);
    
    // Aquí deberías llamar a tu función de escena
    // Este es un placeholder
    return normalize(vec3(
        dist - sdSphere(p - e.xyy, vec3(0.0), 1.0),
        dist - sdSphere(p - e.yxy, vec3(0.0), 1.0),
        dist - sdSphere(p - e.yyx, vec3(0.0), 1.0)
    ));
}

// Anti-aliasing usando fwidth (SDF-AA de ADead-GPU)
float sdfAntialiasing(float dist) {
    float fw = fwidth(dist);
    return smoothstep(-fw, fw, dist);
}

#endif // SDF_PRIMITIVES_GLSL
