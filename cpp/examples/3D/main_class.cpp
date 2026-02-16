// =============================================================================
// REACTOR 3D — Class-Based Example (Recommended Pattern)
// =============================================================================
// This demonstrates the clean C++ SDK pattern using reactor::Application.
// This is the recommended way to build games with REACTOR in C++.
// =============================================================================

#include <reactor/reactor.hpp>
#include <cstdio>

class MiJuego : public reactor::Application {
    float rotacion = 0.0f;
    reactor::MeshHandle* cubo = nullptr;
    reactor::MaterialHandle* material = nullptr;
    int32_t cubo_index = -1;

public:
    // Configuration - called before initialization
    reactor::Config config() override {
        return reactor::Config("REACTOR 3D - Ejemplo de Clase")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4);
    }

    // Initialization - called once at startup
    void on_init() override {
        printf("+==============================================================+\n");
        printf("|           REACTOR 3D - Ejemplo con Clase C++                 |\n");
        printf("+==============================================================+\n");
        printf("\n");
        printf("GPU: %s\n", reactor::GPU::name());
        printf("MSAA: %ux\n", reactor::GPU::msaa_samples());
        printf("\n");
        printf("Controles:\n");
        printf("  ESC - Salir\n");
        printf("  WASD - Mover camara\n");
        printf("\n");

        // Setup camera
        reactor::Camera::set_position({0.0f, 2.0f, 5.0f});
        reactor::Camera::set_target({0.0f, 0.0f, 0.0f});

        // Setup lighting
        reactor::Lighting::add_directional({-0.5f, -1.0f, -0.3f}, {1.0f, 0.98f, 0.95f}, 1.0f);

        // Create cube
        cubo = reactor::Mesh::create_cube();
        if (cubo) {
            printf("Cubo creado!\n");

            // Create material
            material = reactor::Material::create_simple(1.0f, 0.5f, 0.2f);
            if (material) {
                printf("Material creado!\n");

                // Add to scene
                cubo_index = reactor::Scene::add_object(cubo, material, reactor::Mat4::identity());
                printf("Cubo agregado a escena (index: %d)\n", cubo_index);
            }
        }

        printf("Objetos en escena: %u\n", reactor::Scene::object_count());
        printf("REACTOR inicializado!\n");
    }

    // Update - called every frame
    void on_update(float dt) override {
        rotacion += dt;

        // Input handling
        if (reactor::Input::key_pressed(reactor::Input::KEY_ESCAPE())) {
            reactor::Window::request_close();
        }

        // Rotate cube
        if (cubo_index >= 0) {
            reactor::Mat4 transform = reactor::Mat4::rotation_y(rotacion);
            reactor::Scene::set_transform(cubo_index, transform);
        }

        // Show FPS
        if (reactor::Time::frame_count() % 60 == 0) {
            printf("\rFPS: %.1f    ", reactor::Time::fps());
            fflush(stdout);
        }
    }

    // Render - called every frame (optional, scene renders automatically)
    void on_render() override {
        // Custom rendering here if needed
    }

    // Shutdown - called when closing
    void on_shutdown() override {
        printf("\nCerrando REACTOR...\n");
        // Resources are cleaned up automatically
    }
};

// =============================================================================
// MAIN — THE ONE CALL
// =============================================================================

int main() {
    printf("\n");
    printf("Starting REACTOR 3D (Class Pattern)...\n");
    printf("\n");

    // Create and run the game
    return MiJuego().run();
}
