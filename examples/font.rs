#[macro_use] extern crate glium;
extern crate sdf_text;

use std::env;

use glium::{glutin, DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseScrollDelta, TouchPhase};

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
    // Parse args
    let mut args = env::args();
    let font_name = args.nth(1).unwrap_or("assets/FreeSans.ttf".to_string());
    let char_list = args.next().unwrap_or("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".to_string());

    // Build font texture (OpenGL not needed yet)
    let mut font = Font::new(1024);
    font.build_from_file(font_name, 0, 128, 3, char_list.as_str());

    // Create OpenGL window
    let display = glutin::WindowBuilder::new().build_glium().unwrap();

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
    loop {
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
        for event in display.poll_events() {
            match event {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                Event::MouseWheel(delta, TouchPhase::Moved) => {
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            zoom += y * zoom / 4.0;
                            if zoom < 0.01 { zoom = 0.01; }
                        }
                        MouseScrollDelta::PixelDelta(_, _) => (),
                    }
                }
                _ => (),
            }
        }
    }
}
