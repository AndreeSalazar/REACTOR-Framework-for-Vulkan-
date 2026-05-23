//! Gameplay systems
//! 
//! Contains game systems (physics, animation, audio, AI).

pub mod animation;
pub mod audio;
pub mod fps_controller;
pub mod frustum;
pub mod lighting;
pub mod particles;
pub mod physics;
pub mod scene;
pub mod event_bus;

// Re-exports for backward compatibility
pub use animation::{
    AnimationClip, AnimationPlayer, AnimationTrack, EasingFunction, Keyframe, LoopMode, Tween,
};
pub use audio::{AudioClipId, AudioListener, AudioSource, AudioSourceId, AudioSystem};
pub use frustum::{CullingSystem, Frustum, FrustumTestResult, Plane};
pub use lighting::{Light, LightType, LightingSystem};
pub use particles::{EmitterShape, Particle, ParticleSystem, ParticleSystemConfig};
pub use physics::{PhysicsWorld, Ray, RigidBody, Sphere, AABB};
pub use scene::{Scene, SceneObject};
pub use event_bus::{EventBus, Observer};
