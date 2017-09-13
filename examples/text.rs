/* Draw text
 *
 * Controls:
 *   Escape             quit
 *   mouse wheel        zoom in/out
 */

#[macro_use] extern crate glium;
extern crate sdf_text;

use std::env;

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

    const vec3 c_inside = vec3(1.0, 1.0, 1.0);

    void main() {
        float w = texture(tex, v_tex_coords).r;
        float aaw = 0.5 * fwidth(w);
        float alpha = smoothstep(0.50 - aaw, 0.50 + aaw, w);
        if (alpha <= 0.01) {
            discard;
        }
        color = vec4(c_inside, alpha);
    }
"#;

fn main() {
    // Parse args
    let mut args = env::args();
    let font_name = args.nth(1).unwrap_or("assets/FreeSans.ttf".to_string());
    let input_text = args.next().unwrap_or("Hello world!".to_string());

    // Extract set of characters from input text
    let mut char_list: Vec<char> = input_text.chars().collect();
    char_list.sort();
    char_list.dedup();
    let char_list: String = char_list.into_iter().collect();

    // Build font texture (OpenGL not needed yet)
    let face_size = 256;
    let mut font = Font::new(1024);
    font.build_from_file(font_name, 0, face_size, 3, char_list.as_str());

    // Create OpenGL window
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Add a quad for each char into vertex buffer
    let num_chars = input_text.chars().count();
    let mut vertices = Vec::with_capacity(num_chars * 4);
    let mut indices = Vec::with_capacity(num_chars * 6);
    let mut xpos = 0f32;
    for ch in input_text.chars() {
        // Font texture coords
        let glyph_coords = font.glyphs.get(&ch).unwrap();
        let x1 = glyph_coords.x as f32 / font.width as f32 ;
        let y1 = glyph_coords.y as f32 / font.height as f32;
        let x2 = (glyph_coords.x + glyph_coords.width) as f32 / font.width as f32;
        let y2 = (glyph_coords.y + glyph_coords.height) as f32 / font.height as f32;

        // Vertex coords, indices
        // TODO: position
        let vertex1 = Vertex { position: [ -0.6 + xpos, -0.5], tex_coords: [x1, y2] };
        let vertex2 = Vertex { position: [ -0.5 + xpos, -0.5], tex_coords: [x2, y2] };
        let vertex3 = Vertex { position: [ -0.6 + xpos,  0.5], tex_coords: [x1, y1] };
        let vertex4 = Vertex { position: [ -0.5 + xpos,  0.5], tex_coords: [x2, y1] };
        let n = vertices.len() as u16;
        vertices.append(&mut vec![vertex1, vertex2, vertex3, vertex4]);
        indices.append(&mut vec![n, n+1, n+2, n+2, n+1, n+3]);
        xpos += 0.1;
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

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
        blend: glium::Blend::alpha_blending(),
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
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

            target.clear_color(0.0, 0.0, 0.1, 1.0);
            target.draw(&vertex_buffer, &index_buffer, &program,
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
                    },
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
