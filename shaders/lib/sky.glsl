// =============================================================================
// REACTOR · shaders/lib/sky.glsl — Physically-Based Atmospheric Scattering
// =============================================================================
// Simulación analítica de dispersión Rayleigh y Mie para renderizado de cielo.
//   • Rayleigh: dispersión en moléculas de gas (azul del cielo).
//   • Mie: dispersión en partículas grandes/aerosoles (halo blanco del sol).
//   • Ozone: absorción en capas altas (matiz azul/violeta en crepúsculos).
// =============================================================================
#ifndef REACTOR_LIB_SKY
#define REACTOR_LIB_SKY

const vec3 SKY_RAYLEIGH_COEFF = vec3(5.802e-6, 1.3558e-5, 3.31e-5); // Coeficientes para R, G, B
const float SKY_MIE_COEFF = 21.0e-6;
const float SKY_MIE_G = 0.78; // Asimetría de Henyey-Greenstein para el halo solar

// Fase de Rayleigh: dispersión simétrica
float sky_phase_rayleigh(float cosTheta) {
    return 3.0 / (16.0 * 3.14159265) * (1.0 + cosTheta * cosTheta);
}

// Fase de Mie: dispersión hacia adelante (halo solar)
float sky_phase_mie(float cosTheta, float g) {
    float g2 = g * g;
    float denom = 1.0 + g2 - 2.0 * g * cosTheta;
    return 1.0 / (4.0 * 3.14159265) * (1.0 - g2) / (denom * sqrt(denom));
}

// Modelo simplificado de densidad y transmitancia atmosférica
vec3 sky_transmittance(float cosZenith, float turbidity) {
    float depth = 1.0 / max(cosZenith + 0.05, 0.001);
    vec3 opticalDepthRayleigh = SKY_RAYLEIGH_COEFF * depth;
    vec3 opticalDepthMie = vec3(SKY_MIE_COEFF) * depth * turbidity;
    return exp(-(opticalDepthRayleigh + opticalDepthMie));
}

// Evalúa el color del cielo para una dirección de vista dada y dirección solar
// viewDir: vector normalizado apuntando al cielo/horizonte
// sunDir: vector normalizado apuntando hacia el sol
// turbidity: turbidez del aire (1.0 = aire puro, >3.0 = neblina/polvo)
vec3 evaluate_atmosphere(vec3 viewDir, vec3 sunDir, float turbidity) {
    // Asegurar que la dirección de vista esté normalizada
    vec3 V = normalize(viewDir);
    vec3 S = normalize(sunDir);

    float cosTheta = dot(V, S);
    float cosZenithV = max(V.y, 0.0);
    float cosZenithS = max(S.y, 0.0);

    // 1. Transmitancia desde el sol y la vista
    vec3 T_view = sky_transmittance(cosZenithV, turbidity);
    vec3 T_sun = sky_transmittance(cosZenithS, turbidity);

    // 2. Coeficientes de dispersión
    vec3 betaR = SKY_RAYLEIGH_COEFF * turbidity;
    vec3 betaM = vec3(SKY_MIE_COEFF) * turbidity;

    // 3. Funciones de fase
    float phaseR = sky_phase_rayleigh(cosTheta);
    float phaseM = sky_phase_mie(cosTheta, SKY_MIE_G);

    // 4. Inscattering primario (ecuación de dispersión simple)
    vec3 scatterR = betaR * phaseR;
    vec3 scatterM = betaM * phaseM;

    vec3 inscatter = (scatterR + scatterM) / max(betaR + betaM, vec3(1e-5));
    vec3 skyColor = (inscatter * (1.0 - T_view)) * T_sun;

    // 5. Añadir disco solar directo
    float sunIntensity = 24.0;
    float sunAngularDiameterCos = 0.9992; // Aproximadamente 0.5 grados
    if (cosTheta > sunAngularDiameterCos) {
        float sunEdgeBlend = smoothstep(sunAngularDiameterCos, sunAngularDiameterCos + 0.0005, cosTheta);
        skyColor += T_view * sunIntensity * sunEdgeBlend;
    }

    // 6. Tono ambiental del crepúsculo (Ozone / Sky glow)
    float horizonGlow = pow(1.0 - cosZenithV, 4.0);
    skyColor += vec3(0.02, 0.04, 0.08) * T_view * horizonGlow;

    return max(skyColor, vec3(0.0));
}

#endif
