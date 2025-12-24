// REACTOR Systems Module
// Contains game systems (input, ECS, scene management, physics, animation)

pub mod input;
pub mod ecs;
pub mod scene;
pub mod camera;
pub mod transform;
pub mod lighting;
pub mod physics;
pub mod frustum;
pub mod animation;
pub mod audio;
pub mod particles;

pub use input::Input;
pub use ecs::{World, Entity, Component};
pub use scene::{Scene, SceneObject};
pub use camera::{Camera, Camera2D};
pub use transform::Transform;
pub use lighting::{Light, LightType, LightingSystem};
pub use physics::{RigidBody, AABB, Sphere, Ray, PhysicsWorld};
pub use frustum::{Frustum, Plane, CullingSystem, FrustumTestResult};
pub use animation::{AnimationClip, AnimationPlayer, AnimationTrack, Keyframe, LoopMode, Tween, EasingFunction};
pub use audio::{AudioSystem, AudioSource, AudioListener, AudioClipId, AudioSourceId};
pub use particles::{ParticleSystem, Particle, ParticleSystemConfig, EmitterShape};
