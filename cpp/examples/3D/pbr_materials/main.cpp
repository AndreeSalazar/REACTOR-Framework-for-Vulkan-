// =============================================================================
// REACTOR — PBR Materials Example (C++)
// =============================================================================
// Demonstrates the PBR material system:
//   - Creating PBR materials with metallic/roughness workflow
//   - Material instances (inherit + override)
//   - Dynamic parameter tweaking at runtime
//   - Emissive materials
//
// Rust handles GPU pipeline, C++ drives material parameters via shared ABI.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class PBRDemo : public Application {
    PBRMaterial base_metal_;
    PBRMaterial base_plastic_;
    PBRMaterial emissive_mat_;
    PBRMaterial instances_[10];

    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* cube_mat_ = nullptr;
    int32_t cube_indices_[12];
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — PBR Materials Demo", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR PBR Materials Demo ===\n\n");

        // Setup scene
        cube_mesh_ = reactor_create_cube();
        cube_mat_ = reactor_create_material_simple(0.5f, 0.5f, 0.5f);
        reactor_set_camera_position(0, 3, 12);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 0.98f, 0.95f, 1.5f);
        reactor_add_point_light(3, 2, 3, 0.3f, 0.6f, 1.0f, 2.0f, 10.0f);

        // --- Create base PBR materials ---
        base_metal_ = PBRMaterial::create(Vec4(0.9f, 0.9f, 0.95f, 1.0f), 1.0f, 0.2f);
        printf("Metal material created (id=%u)\n", base_metal_.id());

        base_plastic_ = PBRMaterial::create(Vec4(0.8f, 0.2f, 0.1f, 1.0f), 0.0f, 0.6f);
        printf("Plastic material created (id=%u)\n", base_plastic_.id());

        // --- Emissive material ---
        emissive_mat_ = PBRMaterial::create(Vec4(0.1f, 0.1f, 0.1f, 1.0f), 0.0f, 0.9f);
        emissive_mat_.set_emissive(Vec3(0.0f, 1.0f, 0.5f), 5.0f);
        printf("Emissive material created (id=%u)\n", emissive_mat_.id());

        // --- Create material instances with varying roughness ---
        printf("\nCreating roughness gradient (10 instances):\n");
        for (int i = 0; i < 5; ++i) {
            instances_[i] = base_metal_.create_instance();
            float roughness = (float)i / 4.0f;
            instances_[i].set_metallic_roughness(1.0f, roughness);
            printf("  Metal instance %d: roughness=%.2f (id=%u)\n", i, roughness, instances_[i].id());
        }
        for (int i = 0; i < 5; ++i) {
            instances_[5 + i] = base_plastic_.create_instance();
            float roughness = (float)i / 4.0f;
            instances_[5 + i].set_metallic_roughness(0.0f, roughness);
            // Color gradient
            float hue = (float)i / 5.0f;
            instances_[5 + i].set_base_color(Vec4(
                0.5f + 0.5f * sinf(hue * 6.28f),
                0.5f + 0.5f * sinf(hue * 6.28f + 2.09f),
                0.5f + 0.5f * sinf(hue * 6.28f + 4.18f),
                1.0f
            ));
            printf("  Plastic instance %d: roughness=%.2f (id=%u)\n", i, roughness, instances_[5 + i].id());
        }

        // Add cubes to Vulkan scene
        if (cube_mesh_ && cube_mat_) {
            for (int row = 0; row < 2; ++row) {
                for (int col = 0; col < 5; ++col) {
                    CMat4 t{};
                    t.cols[0][0] = 0.8f; t.cols[1][1] = 0.8f;
                    t.cols[2][2] = 0.8f; t.cols[3][3] = 1.0f;
                    t.cols[3][0] = -4.0f + col * 2.0f;
                    t.cols[3][1] = row * 2.5f;
                    t.cols[3][2] = 0.0f;
                    cube_indices_[row * 5 + col] = reactor_add_object(cube_mesh_, cube_mat_, t);
                }
            }
            // Emissive cube
            CMat4 t{};
            t.cols[0][0] = 1.2f; t.cols[1][1] = 1.2f;
            t.cols[2][2] = 1.2f; t.cols[3][3] = 1.0f;
            t.cols[3][0] = 0; t.cols[3][1] = -2.0f; t.cols[3][2] = 0;
            cube_indices_[10] = reactor_add_object(cube_mesh_, cube_mat_, t);
        }

        printf("\nTotal PBR materials: %u\n", PBRMaterial::count());
        printf("Controls: ESC = Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // Animate emissive pulse
        float pulse = (sinf(time_ * 3.0f) + 1.0f) * 0.5f;
        emissive_mat_.set_emissive(
            Vec3(pulse * 0.2f, 1.0f * pulse, 0.5f * pulse),
            2.0f + pulse * 8.0f
        );

        // Rotate cubes
        for (int row = 0; row < 2; ++row) {
            for (int col = 0; col < 5; ++col) {
                int idx = cube_indices_[row * 5 + col];
                if (idx >= 0) {
                    float angle = time_ * 0.5f + col * 0.3f;
                    float ca = cosf(angle), sa = sinf(angle);
                    CMat4 t{};
                    t.cols[0][0] = 0.8f * ca; t.cols[0][2] = 0.8f * sa;
                    t.cols[1][1] = 0.8f;
                    t.cols[2][0] = -0.8f * sa; t.cols[2][2] = 0.8f * ca;
                    t.cols[3][3] = 1.0f;
                    t.cols[3][0] = -4.0f + col * 2.0f;
                    t.cols[3][1] = row * 2.5f;
                    reactor_set_object_transform(idx, t);
                }
            }
        }

        // Rotate emissive cube
        if (cube_indices_[10] >= 0) {
            float a = time_;
            float ca = cosf(a), sa = sinf(a);
            CMat4 t{};
            t.cols[0][0] = 1.2f * ca; t.cols[0][1] = 1.2f * sa;
            t.cols[1][0] = -1.2f * sa; t.cols[1][1] = 1.2f * ca;
            t.cols[2][2] = 1.2f; t.cols[3][3] = 1.0f;
            t.cols[3][1] = -2.0f;
            reactor_set_object_transform(cube_indices_[10], t);
        }

        if (reactor_get_frame_count() % 120 == 0) {
            printf("\rFPS: %.1f | PBR Materials: %u    ", Time::fps(), PBRMaterial::count());
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}

    void on_shutdown() override {
        base_metal_.destroy();
        base_plastic_.destroy();
        emissive_mat_.destroy();
        for (auto& inst : instances_) inst.destroy();
        printf("\nPBR materials cleaned up.\n");
    }
};

int main() {
    PBRDemo app;
    return app.run();
}
