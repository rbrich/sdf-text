/* Example of rendering single glyph
 * Keyboard controls:
 *   Escape             quit
 *   F1                 enable bilinear filtering
 *   F2                 enable SDF shader
 *   numbers, letters   change displayed glyph
 */

#[macro_use] extern crate glium;
extern crate freetype as ft;
extern crate gl_text;

use std::f32;
use glium::{glutin, DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode};

use gl_text::*;


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

const FRAGMENT_SHADER_SDF: &'static str = r#"
    #version 140

    in vec2 v_tex_coords;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        float w = texture(tex, v_tex_coords).r;
        w = smoothstep(0.49, 0.51, w);
        color = vec4(w, w, w, 1.0);
    }
"#;


fn glyph_to_sdf<'a>(c: char, face: &'a ft::Face) -> glium::texture::RawImage2d<'a, u8> {
    // Make SDF texture from the glyph
    face.load_char(c as usize, ft::face::NO_HINTING).unwrap();
    let outline = face.glyph().outline().unwrap();
    let w: u32 = face.glyph().metrics().width as u32 / 64;
    let h: u32 = face.glyph().metrics().height as u32 / 64;
    let mut buffer = Vec::<u8>::with_capacity((w * h) as usize);
    for yr in 0 .. h {
        let y = h - yr - 1;
        for x in 0 .. w {
            let mp = Point::new(x as f32, y as f32);
            let mut dist_min = f32::INFINITY;

            for contour in outline.contours_iter() {
                let mut p0 = Point::new(contour.start().x as f32 / 64.,
                                        contour.start().y as f32 / 64.);
//                println!("{}/{} {}/{}", contour.start().x, face.glyph().metrics().width,
//                                        contour.start().y, face.glyph().metrics().height);
                for curve in contour {
                    let dist;
                    match curve {
                        ft::outline::Curve::Line(a) => {
                            let p1 = Point::new(a.x as f32 / 64., a.y as f32 / 64.);
                            dist = line_distance(mp, p0, p1);
                            p0 = p1;

                        }
                        ft::outline::Curve::Bezier2(a, b) => {
                            let p1 = Point::new(a.x as f32 / 64., a.y as f32 / 64.);
                            let p2 = Point::new(b.x as f32 / 64., b.y as f32 / 64.);
                            dist = quadratic_distance(mp, p0, p1, p2);
                            p0 = p2;
                        }
                        ft::outline::Curve::Bezier3(a, b, c) => {
                            let p1 = Point::new(a.x as f32 / 64., a.y as f32 / 64.);
                            let p2 = Point::new(b.x as f32 / 64., b.y as f32 / 64.);
                            let p3 = Point::new(c.x as f32 / 64., c.y as f32 / 64.);
                            dist = cubic_distance(mp, p0, p1, p2, p3);
                            p0 = p3;
                        }
                    };
                    if dist.abs() < dist_min.abs() {
                        dist_min = dist;
                    }
                }
            }
            // Convert float distance to discrete space (u8):
            // 0 << 191 = outside
            // 192 = zero distance (the outline)
            // 193 >> 256 = inside
            let shift = 128.0;
            let scale = 255.0 / w as f32;
            dist_min = shift - dist_min * scale;
            if dist_min < 0. { dist_min = 0.; }
            if dist_min > 255. { dist_min = 255.; }
            buffer.push(dist_min as u8);
        }
    }
    glium::texture::RawImage2d {
        data: buffer.into(),
        width: w,
        height: h,
        format: glium::texture::ClientFormat::U8,
    }
}

fn glyph_to_image<'a>(c: char, face: &'a ft::Face) -> glium::texture::RawImage2d<'a, u8> {
    // Make texture from the glyph
    face.load_char(c as usize, ft::face::RENDER).unwrap();
    let bitmap = face.glyph().bitmap();
    assert_eq!(bitmap.pixel_mode().unwrap(), ft::bitmap::PixelMode::Gray);
    let buffer = Vec::from(bitmap.buffer());
    glium::texture::RawImage2d {
        data: buffer.into(),
        width: bitmap.width() as u32,
        height: bitmap.rows() as u32,
        format: glium::texture::ClientFormat::U8,
    }
}

fn main() {
    // Create OpenGL window
    let display = glutin::WindowBuilder::new()
        .build_glium().unwrap();

    // Prepare quad
    let vertex1 = Vertex { position: [ -0.5, -0.5], tex_coords: [0.0, 1.0] };
    let vertex2 = Vertex { position: [  0.5, -0.5], tex_coords: [1.0, 1.0] };
    let vertex3 = Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 0.0] };
    let vertex4 = Vertex { position: [  0.5,  0.5], tex_coords: [1.0, 0.0] };
    let quad = vec![vertex1, vertex2, vertex3, vertex4];
    let quad_buffer = glium::VertexBuffer::new(&display, &quad).unwrap();
    let quad_indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    // Prepare shaders and draw params
    let program_direct = match glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None) {
        Ok(res) => res,
        Err(glium::program::ProgramCreationError::CompilationError(err)) => {
            println!("Shader compile error:\n{}", err);
            return;
        },
        Err(other) => panic!(other),
    };
    let program_sdf = match glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER_SDF, None) {
        Ok(res) => res,
        Err(glium::program::ProgramCreationError::CompilationError(err)) => {
            println!("Shader compile error:\n{}", err);
            return;
        },
        Err(other) => panic!(other),
    };
    let mut program = &program_direct;
    let params = glium::DrawParameters {
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    // Load a glyph from font
    let library = ft::Library::init().unwrap();
    let face = library.new_face("assets/GFSDidot.otf", 0).unwrap();
    face.set_pixel_sizes(64, 0).unwrap();
    let face_metrics = face.size_metrics().unwrap();
    let image = glyph_to_sdf('&', &face);
    let mut texture = glium::texture::Texture2d::new(&display, image).unwrap();
    let mut magnify_filter = glium::uniforms::MagnifySamplerFilter::Nearest;

    loop {
        // Draw frame
        {
            let mut target = display.draw();

            // Prepare projection matrix
            let (width, height) = target.get_dimensions();
            let aspect_ratio = width as f32 / height as f32;
            let face_height = face_metrics.ascender - face_metrics.descender;
            let image_width = face.glyph().metrics().width as f32 / face_height as f32;
            let image_height = face.glyph().metrics().height as f32 / face_height as f32;
            let zoom = 2.0;
            let projection = [
                [zoom * image_width / aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, zoom * image_height, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, -0.5, 0.0, 1.0f32],
            ];
            // Model size of the glyph is 1.0 x 1.0
            // font baseline (origin) is at 0.0, ie. screen center
            // the baseline is moved a little down in projection
            let image_y = face.glyph().metrics().horiBearingY as f32  / face.glyph().metrics().height as f32;
            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, -0.5 + image_y, 0.0, 1.0f32],
            ];

            let texture_sampler = glium::uniforms::Sampler::new(&texture)
                        .magnify_filter(magnify_filter)
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

            target.clear_color(0.0, 0.0, 0.1, 1.0);
            target.draw(&quad_buffer, &quad_indices, program,
                        &uniform! { projection: projection, model: model, tex: texture_sampler, },
                        &params).unwrap();
            target.finish().unwrap();
        }
        // Handle events
        for event in display.poll_events() {
            match event {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F1)) => {
                    if magnify_filter == glium::uniforms::MagnifySamplerFilter::Nearest {
                        magnify_filter = glium::uniforms::MagnifySamplerFilter::Linear;
                    } else {
                        magnify_filter = glium::uniforms::MagnifySamplerFilter::Nearest;
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F2)) => {
                    if program as *const _ == &program_direct as *const _ {
                        program = &program_sdf;
                    } else {
                        program = &program_direct;
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {
                    let key = key as u8;
                    let c =
                        if key>= VirtualKeyCode::Key1 as u8 && key <= VirtualKeyCode::Key9 as u8 {
                            ('1' as u8 + (key - VirtualKeyCode::Key1 as u8)) as char
                        } else if key == VirtualKeyCode::Key0 as u8 {
                            '0'
                        } else if key >= VirtualKeyCode::A as u8 && key <= VirtualKeyCode::Z as u8 {
                            ('a' as u8 + (key - VirtualKeyCode::A as u8)) as char
                        } else {
                            '&'
                        };
                    let image = glyph_to_image(c, &face);
                    texture = glium::texture::Texture2d::new(&display, image).unwrap();
                },
                _ => (),
            }
        }
    }
}
