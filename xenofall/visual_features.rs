// =============================================================================
// XENOFALL — Professional Visual Feature Roadmap
// =============================================================================
// This module is intentionally data-first. Before adding more Vulkan passes, the
// test scene needs a shared vocabulary for what each feature costs, how it is
// cached, and which shader/resources it will need.
// =============================================================================

use reactor_vulkan::prelude::*;

#[derive(Clone, Copy)]
pub enum FeaturePriority {
    Phase2Now,
    Phase3Next,
    Phase4Advanced,
}

impl FeaturePriority {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Phase2Now => "Phase 2 / now",
            Self::Phase3Next => "Phase 3 / next",
            Self::Phase4Advanced => "Phase 4 / advanced",
        }
    }
}

#[derive(Clone, Copy)]
pub enum CacheTier {
    Persistent,
    SemiPersistent,
    Dynamic,
    Temporal,
}

impl CacheTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Persistent => "persistent VRAM",
            Self::SemiPersistent => "semi-persistent",
            Self::Dynamic => "dynamic/frame",
            Self::Temporal => "temporal/history",
        }
    }
}

#[derive(Clone, Copy)]
pub struct VisualFeature {
    pub name: &'static str,
    pub priority: FeaturePriority,
    pub cache_tier: CacheTier,
    pub shader_work: &'static str,
    pub reason: &'static str,
}

pub const VISUAL_FEATURES: &[VisualFeature] = &[
    VisualFeature {
        name: "Wet corridor material",
        priority: FeaturePriority::Phase2Now,
        cache_tier: CacheTier::Persistent,
        shader_work: "PBR material tuning: low roughness floor, grime masks, normal detail",
        reason: "The corridor is mostly static, so it is ideal for persistent material/light cache.",
    },
    VisualFeature {
        name: "Water puddles",
        priority: FeaturePriority::Phase2Now,
        cache_tier: CacheTier::SemiPersistent,
        shader_work: "Planar-ish reflection fallback + SSR + animated normal maps + roughness variation",
        reason: "Small puddles give high visual value and stress SSR/TAA without needing full water simulation.",
    },
    VisualFeature {
        name: "Muzzle flash and projectile trails",
        priority: FeaturePriority::Phase2Now,
        cache_tier: CacheTier::Dynamic,
        shader_work: "Additive emissive sprites/mesh billboards, short-lived bloom contributors",
        reason: "Gameplay-critical VFX must remain sharp and should always mark dirty tiles.",
    },
    VisualFeature {
        name: "Impact sparks, blood mist, dust",
        priority: FeaturePriority::Phase2Now,
        cache_tier: CacheTier::Dynamic,
        shader_work: "GPU-friendly particles with depth fade, soft additive/alpha blend",
        reason: "Combat readability improves immediately and gives delta rendering dynamic stress cases.",
    },
    VisualFeature {
        name: "Emergency light flicker",
        priority: FeaturePriority::Phase2Now,
        cache_tier: CacheTier::Dynamic,
        shader_work: "Time-driven light intensity curve, emissive material sync, bloom response",
        reason: "Cheap horror atmosphere and a good test for lighting invalidation.",
    },
    VisualFeature {
        name: "Volumetric fog shafts",
        priority: FeaturePriority::Phase3Next,
        cache_tier: CacheTier::Temporal,
        shader_work: "Low-res froxel/fog pass, bilateral upsample, temporal accumulation",
        reason: "Fog sells depth but must be temporally stable before delta rendering can trust history.",
    },
    VisualFeature {
        name: "Contact shadows + blob shadow refinement",
        priority: FeaturePriority::Phase3Next,
        cache_tier: CacheTier::Dynamic,
        shader_work: "Screen-space contact shadow resolve plus existing blob shadow fallback",
        reason: "Enemies need grounded silhouettes; only dynamic actors should invalidate these regions.",
    },
    VisualFeature {
        name: "Decals: bullet holes, blood, leaks",
        priority: FeaturePriority::Phase3Next,
        cache_tier: CacheTier::SemiPersistent,
        shader_work: "Deferred/projected decals with material ID and lifetime policy",
        reason: "Decals create persistent world memory and are perfect for testing cache invalidation.",
    },
    VisualFeature {
        name: "Sector static lighting cache",
        priority: FeaturePriority::Phase3Next,
        cache_tier: CacheTier::Persistent,
        shader_work: "Static irradiance/probe cache per corridor sector",
        reason: "This is the bridge between the current renderer and 'VRAM that remembers'.",
    },
    VisualFeature {
        name: "Dirty tile debug overlay",
        priority: FeaturePriority::Phase3Next,
        cache_tier: CacheTier::Temporal,
        shader_work: "Compute tile classification + debug full-screen overlay",
        reason: "Delta rendering cannot be trusted until reused vs recalculated regions are visible.",
    },
    VisualFeature {
        name: "Reactive water ripples",
        priority: FeaturePriority::Phase4Advanced,
        cache_tier: CacheTier::Temporal,
        shader_work: "Small ripple simulation texture updated by footsteps/bullets, sampled by water shader",
        reason: "Useful after base puddles exist; otherwise it is over-engineering too early.",
    },
    VisualFeature {
        name: "Selective ray queries",
        priority: FeaturePriority::Phase4Advanced,
        cache_tier: CacheTier::Dynamic,
        shader_work: "Ray queries only for important shadows/reflections, denoised temporally",
        reason: "Should come after raster/SSR/TAA metrics prove where rays are actually needed.",
    },
];

pub fn log_visual_feature_roadmap() {
    Log::section("XENOFALL Visual Roadmap — Shaders, Water, VFX");

    for feature in VISUAL_FEATURES {
        Log::kv(
            feature.name,
            &format!(
                "{} | {} | {}",
                feature.priority.label(),
                feature.cache_tier.label(),
                feature.shader_work
            ),
        );
    }

    Log::kv(
        "Recommended order",
        "materials/puddles/VFX first, then fog/decals/cache, then reactive water/ray queries",
    );
}
