use reactor::{Reactor, Vertex};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::sync::Arc;
use glam::Vec3;

// Minimal Shaders (SPIR-V)
// These would normally be loaded from files, but for "Very Easy" demo we inline them or load them.
// Since we don't have a shader compiler at runtime yet, we need precompiled SPIR-V.
// I'll create a helper to load them or use a byte array.
// For this example, I will assume we have `vert.spv` and `frag.spv` or I can embed them.
// To avoid compilation issues, I will create a dummy triangle shader.

// Wait, I need actual SPIR-V code.
// I'll use a hardcoded SPIR-V for a simple triangle if possible, or I need to ask the user to compile shaders.
// Better: I'll use `inline_spirv` crate if I could, but adding more deps might be risky.
// Let's just use a placeholder or assume the user has shaders.
// Actually, I'll generate a simple triangle shader using a tool or just provide the bytes.
// Since I can't easily generate SPIR-V here, I will check if I can use a library or just raw bytes.
// I'll use a very simple precompiled SPIR-V for a triangle.

// VERTEX SHADER (GLSL)
/*
#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 uv;
layout(location = 0) out vec3 fragColor;
void main() {
    gl_Position = vec4(position, 1.0);
    fragColor = color;
}
*/

// FRAGMENT SHADER (GLSL)
/*
#version 450
layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;
void main() {
    outColor = vec4(fragColor, 1.0);
}
*/

// I will use `ash`'s util to load if files exist, or panic.
// But to make it "Very Easy", I should probably include a default shader in the library?
// No, let's just make the sandbox load them.

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(event_loop.create_window(
        Window::default_attributes()
            .with_title("REACTOR Sandbox (Rust)")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
    ).unwrap());

    let mut reactor = Reactor::init(&window).expect("Failed to initialize Reactor");

    println!("REACTOR initialized successfully with Vulkan!");

    // Define Geometry (Declarative-ish)
    let vertices = [
        Vertex::new(Vec3::new(0.0, -0.5, 0.0), Vec3::new(1.0, 0.0, 0.0), glam::Vec2::ZERO),
        Vertex::new(Vec3::new(0.5, 0.5, 0.0), Vec3::new(0.0, 1.0, 0.0), glam::Vec2::ZERO),
        Vertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec3::new(0.0, 0.0, 1.0), glam::Vec2::ZERO),
    ];
    let indices = [0, 1, 2];

    let mesh = reactor.create_mesh(&vertices, &indices).expect("Failed to create mesh");

    // Load Shaders (Assumes shaders exist, if not, it will fail)
    // For this demo to work immediately, I need to provide the SPIR-V.
    // I will write a simple "build.rs" or just ask to run a script?
    // No, I'll just put the bytes here if I can.
    // It's too long.
    // I'll assume `shaders/vert.spv` and `shaders/frag.spv` exist.
    
    // Let's create a "shaders" folder and put dummy files or instructions?
    // The user wants "Automated TODO".
    // I should probably compile shaders if I can.
    // I can't compile GLSL to SPIR-V easily without `shaderc` crate.
    // I'll add `shaderc` to dependencies? It requires C++ build tools.
    // Maybe `naga`?
    // Let's stick to reading files. I'll create a task to compile shaders.

    // Temporary: Read from file, expecting user to have them or I'll create them.
    // Actually, I'll read from "shaders/shader.vert.spv" etc.
    
    let vert_code = include_bytes!("../shaders/vert.spv");
    let frag_code = include_bytes!("../shaders/frag.spv");

    let vert_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&vert_code[..])).expect("Failed to read vert spv");
    let frag_decoded = ash::util::read_spv(&mut std::io::Cursor::new(&frag_code[..])).expect("Failed to read frag spv");

    let material = reactor.create_material(&vert_decoded, &frag_decoded).expect("Failed to create material");

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Closing...");
                elwt.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if let Err(e) = reactor.draw_frame(&mesh, &material) {
                    eprintln!("Failed to draw frame: {}", e);
                    elwt.exit();
                }
            }
            _ => (),
        }
    }).unwrap();
}
