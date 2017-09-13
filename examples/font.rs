/* Render font texture
 *
 * Controls:
 *   Escape             quit
 *   mouse wheel        zoom in/out
 */

#[macro_use] extern crate glium;
extern crate sdf_text;

use std::env;
use std::time;

use glium::{glutin, Surface};
use glium::glutin::{Event, WindowEvent, ElementState, VirtualKeyCode, MouseScrollDelta, TouchPhase};

use sdf_text::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

const VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    uniform mat4 projection;
    uniform mat4 model;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = projection * model * vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    #version 140

    in vec2 v_tex_coords;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        float w = texture(tex, v_tex_coords).r;
        color = vec4(w, w, w, 1.0);
    }
"#;

fn main() {
    // Generate default table of characters (all printable ASCII chars)
    let printable_ascii: Vec<u8> = (0x20u8 .. 0x7Eu8).collect();
    let printable_ascii = std::str::from_utf8(&printable_ascii).unwrap();

    // Parse args
    let mut args = env::args();
    let font_name = args.nth(1).unwrap_or("assets/FreeSans.ttf".to_string());
    let char_list = args.next().unwrap_or(printable_ascii.to_string());

    // Build font texture (OpenGL not needed yet)
    let face_size = 128;
    let mut font = Font::new(1024);
    let t_start = time::Instant::now();
    font.build_from_file(font_name, 0, face_size, 3, char_list.as_str());
    let t_end = time::Instant::now();
    let d = t_end.duration_since(t_start);
    println!("Render font texture: face size {} in {}s",
             face_size, d.as_secs() as f32 + d.subsec_nanos() as f32 / 1e9);

    // Create OpenGL window
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Prepare quad
    let vertex1 = Vertex { position: [ -0.5, -0.5], tex_coords: [0.0, 1.0] };
    let vertex2 = Vertex { position: [  0.5, -0.5], tex_coords: [1.0, 1.0] };
    let vertex3 = Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 0.0] };
    let vertex4 = Vertex { position: [  0.5,  0.5], tex_coords: [1.0, 0.0] };
    let quad = vec![vertex1, vertex2, vertex3, vertex4];
    let quad_buffer = glium::VertexBuffer::new(&display, &quad).unwrap();
    let quad_indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    // Prepare shaders and draw params
    let program = match glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None) {
        Ok(res) => res,
        Err(glium::program::ProgramCreationError::CompilationError(err)) => {
            println!("Shader compile error:\n{}", err);
            return;
        },
        Err(other) => panic!(other),
    };
    let params = glium::DrawParameters {
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    // Transform font texture to GL
    let image = glium::texture::RawImage2d {
        data: font.buffer.into(),
        width: font.width as u32,
        height: font.height as u32,
        format: glium::texture::ClientFormat::U8,
    };
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let mut zoom = 2.0;
    let mut quit = false;
    while !quit {
        // Draw frame
        {
            let mut target = display.draw();

            // Prepare projection matrix
            let (width, height) = target.get_dimensions();
            let aspect_ratio = width as f32 / height as f32;
            let projection = [
                [zoom / aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, zoom, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ];
            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ];

            let texture_sampler = glium::uniforms::Sampler::new(&texture)
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

            target.clear_color(0.0, 0.0, 0.1, 1.0);
            target.draw(&quad_buffer, &quad_indices, &program,
                        &uniform! { projection: projection, model: model, tex: texture_sampler, },
                        &params).unwrap();
            target.finish().unwrap();
        }
        // Handle events
        events_loop.poll_events(|event|
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => quit = true,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed {
                            match input.virtual_keycode {
                                Some(VirtualKeyCode::Escape) => quit = true,
                                _ => ()
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, phase: TouchPhase::Moved, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(_, y) => {
                                zoom += y * zoom / 4.0;
                                if zoom < 0.01 { zoom = 0.01; }
                            }
                            MouseScrollDelta::PixelDelta(_, y) => {
                                zoom += y * zoom / 40.0;
                                if zoom < 0.01 { zoom = 0.01; }
                            }
                        }
                    },
                    _ => ()
                },
                _ => ()
            }
        );
    }
}
