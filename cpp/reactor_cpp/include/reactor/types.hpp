// =============================================================================
// REACTOR C++ SDK — Shared Types
// =============================================================================
// Math types that mirror Rust's glam types.
// These are repr(C) compatible with the C API.
// Ultra-productive, ultra-powerful math library.
// =============================================================================

#pragma once

#include <cmath>
#include <cstdint>
#include "core.hpp"

namespace reactor {

// =============================================================================
// Vec2 — 2D Vector
// =============================================================================

struct Vec2 {
    float x, y;

    Vec2() : x(0), y(0) {}
    Vec2(float x, float y) : x(x), y(y) {}
    Vec2(float v) : x(v), y(v) {}
    Vec2(const CVec2& c) : x(c.x), y(c.y) {}

    operator CVec2() const { return {x, y}; }

    Vec2 operator+(const Vec2& o) const { return {x+o.x, y+o.y}; }
    Vec2 operator-(const Vec2& o) const { return {x-o.x, y-o.y}; }
    Vec2 operator*(float s) const { return {x*s, y*s}; }
    Vec2 operator*(const Vec2& o) const { return {x*o.x, y*o.y}; }
    Vec2 operator/(float s) const { return {x/s, y/s}; }
    Vec2 operator-() const { return {-x, -y}; }

    Vec2& operator+=(const Vec2& o) { x+=o.x; y+=o.y; return *this; }
    Vec2& operator-=(const Vec2& o) { x-=o.x; y-=o.y; return *this; }
    Vec2& operator*=(float s) { x*=s; y*=s; return *this; }

    float dot(const Vec2& o) const { return x*o.x + y*o.y; }
    float length() const { return std::sqrt(x*x + y*y); }
    float length_squared() const { return x*x + y*y; }
    Vec2 normalized() const { float l = length(); return l > 0 ? Vec2{x/l, y/l} : Vec2{}; }

    static Vec2 Zero() { return {0, 0}; }
    static Vec2 One() { return {1, 1}; }
};

// =============================================================================
// Vec3 — 3D Vector
// =============================================================================

struct Vec3 {
    float x, y, z;

    Vec3() : x(0), y(0), z(0) {}
    Vec3(float x, float y, float z) : x(x), y(y), z(z) {}
    Vec3(float v) : x(v), y(v), z(v) {}
    Vec3(const CVec3& c) : x(c.x), y(c.y), z(c.z) {}

    operator CVec3() const { return {x, y, z}; }

    Vec3 operator+(const Vec3& o) const { return {x+o.x, y+o.y, z+o.z}; }
    Vec3 operator-(const Vec3& o) const { return {x-o.x, y-o.y, z-o.z}; }
    Vec3 operator*(float s) const { return {x*s, y*s, z*s}; }
    Vec3 operator*(const Vec3& o) const { return {x*o.x, y*o.y, z*o.z}; }
    Vec3 operator/(float s) const { return {x/s, y/s, z/s}; }
    Vec3 operator-() const { return {-x, -y, -z}; }

    Vec3& operator+=(const Vec3& o) { x+=o.x; y+=o.y; z+=o.z; return *this; }
    Vec3& operator-=(const Vec3& o) { x-=o.x; y-=o.y; z-=o.z; return *this; }
    Vec3& operator*=(float s) { x*=s; y*=s; z*=s; return *this; }

    float dot(const Vec3& o) const { return x*o.x + y*o.y + z*o.z; }
    Vec3 cross(const Vec3& o) const {
        return {y*o.z - z*o.y, z*o.x - x*o.z, x*o.y - y*o.x};
    }
    float length() const { return std::sqrt(x*x + y*y + z*z); }
    float length_squared() const { return x*x + y*y + z*z; }
    Vec3 normalized() const { float l = length(); return l > 0 ? Vec3{x/l, y/l, z/l} : Vec3{}; }

    static Vec3 lerp(const Vec3& a, const Vec3& b, float t) {
        return a + (b - a) * t;
    }

    static Vec3 Zero() { return {0, 0, 0}; }
    static Vec3 One() { return {1, 1, 1}; }
    static Vec3 Up() { return {0, 1, 0}; }
    static Vec3 Down() { return {0, -1, 0}; }
    static Vec3 Forward() { return {0, 0, -1}; }
    static Vec3 Back() { return {0, 0, 1}; }
    static Vec3 Right() { return {1, 0, 0}; }
    static Vec3 Left() { return {-1, 0, 0}; }
};

inline Vec3 operator*(float s, const Vec3& v) { return v * s; }

// =============================================================================
// Vec4 — 4D Vector / Color
// =============================================================================

struct Vec4 {
    float x, y, z, w;

    Vec4() : x(0), y(0), z(0), w(0) {}
    Vec4(float x, float y, float z, float w) : x(x), y(y), z(z), w(w) {}
    Vec4(float v) : x(v), y(v), z(v), w(v) {}
    Vec4(const Vec3& v, float w) : x(v.x), y(v.y), z(v.z), w(w) {}
    Vec4(const CVec4& c) : x(c.x), y(c.y), z(c.z), w(c.w) {}

    operator CVec4() const { return {x, y, z, w}; }

    Vec4 operator+(const Vec4& o) const { return {x+o.x, y+o.y, z+o.z, w+o.w}; }
    Vec4 operator-(const Vec4& o) const { return {x-o.x, y-o.y, z-o.z, w-o.w}; }
    Vec4 operator*(float s) const { return {x*s, y*s, z*s, w*s}; }

    Vec3 xyz() const { return {x, y, z}; }
    Vec3 rgb() const { return {x, y, z}; }

    static Vec4 White() { return {1, 1, 1, 1}; }
    static Vec4 Black() { return {0, 0, 0, 1}; }
    static Vec4 Red() { return {1, 0, 0, 1}; }
    static Vec4 Green() { return {0, 1, 0, 1}; }
    static Vec4 Blue() { return {0, 0, 1, 1}; }
    static Vec4 Yellow() { return {1, 1, 0, 1}; }
    static Vec4 Cyan() { return {0, 1, 1, 1}; }
    static Vec4 Magenta() { return {1, 0, 1, 1}; }
    static Vec4 Clear() { return {0, 0, 0, 0}; }
};

using Color = Vec4;

// =============================================================================
// Mat4 — 4x4 Matrix (Column-major, Vulkan-compatible)
// =============================================================================

struct Mat4 {
    float cols[4][4];

    Mat4() { *this = Identity(); }
    Mat4(const CMat4& c) { std::memcpy(cols, c.cols, sizeof(cols)); }

    operator CMat4() const {
        CMat4 c;
        std::memcpy(c.cols, cols, sizeof(cols));
        return c;
    }

    Mat4 operator*(const Mat4& o) const {
        CMat4 result = reactor_mat4_mul(*this, o);
        return Mat4(result);
    }

    Vec4 operator*(const Vec4& v) const {
        Vec4 result;
        result.x = cols[0][0]*v.x + cols[1][0]*v.y + cols[2][0]*v.z + cols[3][0]*v.w;
        result.y = cols[0][1]*v.x + cols[1][1]*v.y + cols[2][1]*v.z + cols[3][1]*v.w;
        result.z = cols[0][2]*v.x + cols[1][2]*v.y + cols[2][2]*v.z + cols[3][2]*v.w;
        result.w = cols[0][3]*v.x + cols[1][3]*v.y + cols[2][3]*v.z + cols[3][3]*v.w;
        return result;
    }

    Mat4 inverse() const { return Mat4(reactor_mat4_inverse(*this)); }
    Mat4 transpose() const { return Mat4(reactor_mat4_transpose(*this)); }

    static Mat4 Identity() { return Mat4(reactor_mat4_identity()); }

    static Mat4 Translation(float x, float y, float z) {
        return Mat4(reactor_mat4_translation(x, y, z));
    }
    static Mat4 Translation(const Vec3& v) {
        return Translation(v.x, v.y, v.z);
    }

    static Mat4 RotationX(float radians) {
        return Mat4(reactor_mat4_rotation_x(radians));
    }
    static Mat4 RotationY(float radians) {
        return Mat4(reactor_mat4_rotation_y(radians));
    }
    static Mat4 RotationZ(float radians) {
        return Mat4(reactor_mat4_rotation_z(radians));
    }

    static Mat4 Scale(float x, float y, float z) {
        return Mat4(reactor_mat4_scale(x, y, z));
    }
    static Mat4 Scale(const Vec3& v) {
        return Scale(v.x, v.y, v.z);
    }
    static Mat4 Scale(float s) {
        return Scale(s, s, s);
    }

    static Mat4 Perspective(float fov_degrees, float aspect, float near_plane, float far_plane) {
        return Mat4(reactor_mat4_perspective(fov_degrees, aspect, near_plane, far_plane));
    }

    static Mat4 LookAt(const Vec3& eye, const Vec3& target, const Vec3& up = Vec3::Up()) {
        return Mat4(reactor_mat4_look_at(eye, target, up));
    }
};

// =============================================================================
// Transform — Position, Rotation, Scale
// =============================================================================

struct Transform {
    Vec3 position;
    Vec3 rotation;  // Euler angles in radians
    Vec3 scale;

    Transform()
        : position(0, 0, 0)
        , rotation(0, 0, 0)
        , scale(1, 1, 1) {}

    Transform(const Vec3& pos)
        : position(pos)
        , rotation(0, 0, 0)
        , scale(1, 1, 1) {}

    Transform(const Vec3& pos, const Vec3& rot, const Vec3& scl)
        : position(pos)
        , rotation(rot)
        , scale(scl) {}

    Mat4 matrix() const {
        return Mat4::Translation(position)
             * Mat4::RotationY(rotation.y)
             * Mat4::RotationX(rotation.x)
             * Mat4::RotationZ(rotation.z)
             * Mat4::Scale(scale);
    }

    Vec3 forward() const {
        float cy = std::cos(rotation.y);
        float sy = std::sin(rotation.y);
        float cx = std::cos(rotation.x);
        float sx = std::sin(rotation.x);
        return Vec3(-sy * cx, sx, -cy * cx).normalized();
    }

    Vec3 right() const {
        float cy = std::cos(rotation.y);
        float sy = std::sin(rotation.y);
        return Vec3(cy, 0, -sy);
    }

    Vec3 up() const {
        return right().cross(forward());
    }
};

// =============================================================================
// Utility functions
// =============================================================================

inline float lerp(float a, float b, float t) { return reactor_lerp(a, b, t); }
inline float clamp(float v, float min, float max) { return reactor_clamp(v, min, max); }
inline float smoothstep(float edge0, float edge1, float x) { return reactor_smoothstep(edge0, edge1, x); }
inline float deg_to_rad(float degrees) { return reactor_deg_to_rad(degrees); }
inline float rad_to_deg(float radians) { return reactor_rad_to_deg(radians); }

constexpr float PI = 3.14159265358979323846f;
constexpr float TAU = PI * 2.0f;
constexpr float DEG2RAD = PI / 180.0f;
constexpr float RAD2DEG = 180.0f / PI;

} // namespace reactor
