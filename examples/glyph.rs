/* Example of rendering single glyph
 * Keyboard controls:
 *   Escape             quit
 *   F1                 enable bilinear filtering
 *   F2                 enable SDF shader
 *   F3                 render SDF / freetype monochrome texture
 *   numbers, letters   change displayed glyph
 */

#[macro_use] extern crate glium;
extern crate freetype as ft;
extern crate sdf_text;

use std::f32;
use std::time;
use std::env;
use glium::{glutin, DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode};

use sdf_text::*;

pub fn vec2_from_ft(p: ft::Vector, unit: f32) -> Vec2 {
    Vec2 { x: p.x as f32 / unit, y: p.y as f32 / unit }
}

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

    const vec3 c_inside = vec3(1.0, 1.0, 1.0);
    const vec3 c_outline = vec3(0.6, 0.6, 0.0);
    const vec3 c_outside = vec3(0.0, 0.0, 0.0);

    void main() {
        float w = texture(tex, v_tex_coords).r;
        vec3 c_mixed = mix(c_outline, c_inside, smoothstep(0.59, 0.60, w));
        float alpha = smoothstep(0.50, 0.51, w);
        color = vec4(mix(c_outside, c_mixed, alpha), 1.0);
    }
"#;

const PADDING: u32 = 1;
const FACE_SIZE: u32 = 128;

fn glyph_to_sdf<'a>(c: char, face: &'a ft::Face) -> glium::texture::RawImage2d<'a, u8> {
    // Make SDF texture from the glyph
    let t_start = time::Instant::now();
    face.set_pixel_sizes(face.em_size() as u32, 0).unwrap();
    face.load_char(c as usize, ft::face::NO_HINTING).unwrap();
    let outline = face.glyph().outline().unwrap();
    let bbox = face.glyph().get_glyph().unwrap().get_cbox(0);
    let pxsize = face.em_size() as f32 * 64. / FACE_SIZE as f32;
    let xmin = (bbox.xMin as f32 / pxsize).round();
    let ymin = (bbox.yMin as f32 / pxsize).round();
    let xmax = (bbox.xMax as f32 / pxsize).round();
    let ymax = (bbox.yMax as f32 / pxsize).round();
    let w = ((xmax - xmin) + 2.0 * PADDING as f32) as u32;
    let h = ((ymax - ymin) + 2.0 * PADDING as f32) as u32;
    let origin = Vec2::new((xmin - PADDING as f32 +0.5),
                           (ymin - PADDING as f32 +0.5));
    let mut buffer = Vec::<u8>::with_capacity((w * h) as usize);
    // Reversed contour orientation (counter-clockwise filled)
    let outline_flags = face.glyph().raw().outline.flags;
    let reverse_fill = (outline_flags & 0x4) == 0x4; // FT_OUTLINE_REVERSE_FILL;

    // Find intersection points for the scan line
    // (edge crossings algorithm)
    let mut rasterizer = Rasterizer::new();
    for contour in outline.contours_iter() {
        let mut p0 = vec2_from_ft(contour.start(), pxsize);
        for curve in contour {
            match curve {
                ft::outline::Curve::Line(a) => {
                    let p1 = vec2_from_ft(a, pxsize);
                    rasterizer.push_line(p0, p1);
                    p0 = p1;
                }
                ft::outline::Curve::Bezier2(a, b) => {
                    let p1 = vec2_from_ft(a, pxsize);
                    let p2 = vec2_from_ft(b, pxsize);
                    rasterizer.push_bezier2(p0, p1, p2);
                    p0 = p2;
                }
                ft::outline::Curve::Bezier3(a, b, c) => {
                    let p1 = vec2_from_ft(a, pxsize);
                    let p2 = vec2_from_ft(b, pxsize);
                    let p3 = vec2_from_ft(c, pxsize);
                    rasterizer.push_bezier3(p0, p1, p1, p2);
                    p0 = p3;
                }
            };
        }
    }

    for yr in (0..h).rev() {
        let y = origin.y + yr as f32;

        let ref mut crossings = rasterizer.scanline_crossings(y);

        // Find point distance
        let mut crossings_idx = 0;
        let mut wn = 0i32;
        for xr in 0 .. w {
            let x = origin.x + xr as f32;
            let mp = Vec2::new(x, y);
            let mut dist_min = f32::INFINITY;

            // Is the point inside curve?
            while crossings.len() > crossings_idx && crossings[crossings_idx].x <= x {
                wn += crossings[crossings_idx].dir as i32;
                crossings_idx += 1;
            }
            let inside = if reverse_fill { wn < 0 } else { wn > 0 };

            for contour in outline.contours_iter() {
                let mut p0 = vec2_from_ft(contour.start(), pxsize);
                for curve in contour {
                    let dist;
                    match curve {
                        ft::outline::Curve::Line(a) => {
                            let p1 = vec2_from_ft(a, pxsize);
                            dist = line_distance(mp, p0, p1);
                            p0 = p1;
                        }
                        ft::outline::Curve::Bezier2(a, b) => {
                            let p1 = vec2_from_ft(a, pxsize);
                            let p2 = vec2_from_ft(b, pxsize);
                            dist = quadratic_distance(mp, p0, p1, p2);
                            p0 = p2;
                        }
                        ft::outline::Curve::Bezier3(a, b, c) => {
                            let p1 = vec2_from_ft(a, pxsize);
                            let p2 = vec2_from_ft(b, pxsize);
                            let p3 = vec2_from_ft(c, pxsize);
                            dist = cubic_distance(mp, p0, p1, p2, p3);
                            p0 = p3;
                        }
                    };
                    if dist < dist_min {
                        dist_min = dist;
                    }
                }
            }

            if inside {
                dist_min = -dist_min;
            }

            // Convert float distance to discrete space (u8):
            // 0 << 127 = outside
            // 127 = zero distance (the outline)
            // 128 >> 255 = inside
            let shift = 127.0;
            let scale = 1920. / FACE_SIZE as f32;
            dist_min = shift - dist_min * scale;
            if dist_min < 0. { dist_min = 0.; }
            if dist_min > 255. { dist_min = 255.; }
            buffer.push(dist_min as u8);
//            buffer.push(inside as u8 * 255u8);
        }
    }
    face.set_pixel_sizes(FACE_SIZE, 0).unwrap();
    let t_end = time::Instant::now();
    let d = t_end.duration_since(t_start);
    println!("Render: size {}x{} in {}s (SDF)",
             w, h, d.as_secs() as f32 + d.subsec_nanos() as f32 / 1e9);
    glium::texture::RawImage2d {
        data: buffer.into(),
        width: w as u32,
        height: h as u32,
        format: glium::texture::ClientFormat::U8,
    }
}

fn glyph_to_image<'a>(c: char, face: &'a ft::Face) -> glium::texture::RawImage2d<'a, u8> {
    // Make texture from the glyph
    let t_start = time::Instant::now();
    face.set_pixel_sizes(FACE_SIZE, 0).unwrap();
    face.load_char(c as usize, ft::face::RENDER | ft::face::NO_HINTING | ft::face::MONOCHROME).unwrap();
    let bitmap = face.glyph().bitmap();
    assert_eq!(bitmap.pixel_mode().unwrap(), ft::bitmap::PixelMode::Mono);
    assert!(bitmap.pitch() > 0);
    let w = bitmap.width() as u32 + 2*PADDING;
    let h = bitmap.rows() as u32 + 2*PADDING;
    let mut buffer = Vec::<u8>::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            if x >= PADDING && y >= PADDING && x < w - PADDING && y < h - PADDING {
                let i = (y - PADDING) * bitmap.pitch() as u32 + ((x - PADDING) >> 3);
                let b = bitmap.buffer()[i as usize] << ((x - PADDING) % 8);
                buffer.push((b & 0x80) / 128 * 255);
            } else {
                buffer.push(0);
            }
        }
    }
    let t_end = time::Instant::now();
    let d = t_end.duration_since(t_start);
    println!("Render: size {}x{} in {}s (FreeType)",
             w, h, d.as_secs() as f32 + d.subsec_nanos() as f32 / 1e9);
    glium::texture::RawImage2d {
        data: buffer.into(),
        width: w,
        height: h,
        format: glium::texture::ClientFormat::U8,
    }
}

fn main() {
    // Parse args
    let mut args = env::args();
    let font_name = args.nth(1).unwrap_or("assets/FreeSans.ttf".to_string());
    let text_to_show = args.next().unwrap_or("0".to_string());

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
    let mut program = &program_sdf;
    let params = glium::DrawParameters {
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    // Load a glyph from font
    let library = ft::Library::init().unwrap();
    let face = library.new_face(font_name, 0).unwrap();
    face.set_pixel_sizes(FACE_SIZE, 0).unwrap();
    let face_metrics = face.size_metrics().unwrap();
    let mut glyph_char = text_to_show.chars().next().unwrap();
    let image = glyph_to_sdf(glyph_char, &face);
    let mut image_w = image.width;
    let mut image_h = image.height;
    let mut texture = glium::texture::Texture2d::new(&display, image).unwrap();
    let mut magnify_filter = glium::uniforms::MagnifySamplerFilter::Linear;
    let mut sdf = true;
    loop {
        // Draw frame
        {
            let mut target = display.draw();

            // Prepare projection matrix
            let (width, height) = target.get_dimensions();
            let aspect_ratio = width as f32 / height as f32;
            let face_height = face_metrics.ascender - face_metrics.descender;
            let image_width = image_w as f32 / (face_height as f32 / 64.);
            let image_height = image_h as f32 / (face_height as f32 / 64.);
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
                        println!("Magnify filter: Linear");
                        magnify_filter = glium::uniforms::MagnifySamplerFilter::Linear;
                    } else {
                        println!("Magnify filter: Nearest");
                        magnify_filter = glium::uniforms::MagnifySamplerFilter::Nearest;
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F2)) => {
                    if program as *const _ == &program_direct as *const _ {
                        println!("Shader: SDF");
                        program = &program_sdf;
                    } else {
                        println!("Shader: Direct");
                        program = &program_direct;
                    }
                },
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F3)) => {
                    sdf = !sdf;
                    let image = if sdf {
                        glyph_to_sdf(glyph_char, &face)
                    } else {
                        glyph_to_image(glyph_char, &face)
                    };
                    image_w = image.width;
                    image_h = image.height;
                    texture = glium::texture::Texture2d::new(&display, image).unwrap();
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {
                    let key = key as u8;
                    glyph_char =
                        if key>= VirtualKeyCode::Key1 as u8 && key <= VirtualKeyCode::Key9 as u8 {
                            ('1' as u8 + (key - VirtualKeyCode::Key1 as u8)) as char
                        } else if key == VirtualKeyCode::Key0 as u8 {
                            '0'
                        } else if key >= VirtualKeyCode::A as u8 && key <= VirtualKeyCode::Z as u8 {
                            ('a' as u8 + (key - VirtualKeyCode::A as u8)) as char
                        } else {
                            '&'
                        };
                    let image = if sdf {
                        glyph_to_sdf(glyph_char, &face)
                    } else {
                        glyph_to_image(glyph_char, &face)
                    };
                    image_w = image.width;
                    image_h = image.height;
                    texture = glium::texture::Texture2d::new(&display, image).unwrap();
                },
                _ => (),
            }
        }
    }
}
