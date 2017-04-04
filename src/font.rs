use std::path;
use std::collections::HashMap;
use freetype as ft;
use rect_packer;

use rasterizer::*;
use mindist::*;
use curve::*;

pub fn vec2_from_ft(p: ft::Vector, unit: f32) -> Vec2 {
    Vec2 { x: p.x as f32 / unit, y: p.y as f32 / unit }
}

#[derive(Debug)]
pub struct Glyph {
    // coordinates in font texture (top left corner)
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    // metrics
    pub xmin: isize,
    pub ymin: isize,
}

impl Glyph {
    pub fn from_face(face: &ft::Face, face_size: usize,
                     padding: usize) -> Self {
        let bbox = face.glyph().get_glyph().unwrap().get_cbox(0);
        let unit_size = face.em_size() as f32 * 64. / face_size as f32;
        let xmin = (bbox.xMin as f32 / unit_size + 0.5).floor();
        let ymin = (bbox.yMin as f32 / unit_size + 0.5).floor();
        let xmax = (bbox.xMax as f32 / unit_size + 0.5).floor();
        let ymax = (bbox.yMax as f32 / unit_size + 0.5).floor();
        Glyph {
            x: 0,
            y: 0,
            width: (xmax - xmin) as usize + 2 * padding,
            height: (ymax - ymin) as usize + 2 * padding,
            xmin: xmin as isize - padding as isize,
            ymin: ymin as isize - padding as isize,
        }
    }

    pub fn render_sdf(&self, face: &ft::Face, face_size: usize,
                      buffer: &mut [u8], pitch: usize) {
        let outline = face.glyph().outline().unwrap();
        let outline_flags = face.glyph().raw().outline.flags;
        let unit_size = face.em_size() as f32 * 64. / face_size as f32;

        // Reversed contour orientation (counter-clockwise filled)
        let reverse_fill = (outline_flags & 0x4) == 0x4; // FT_OUTLINE_REVERSE_FILL;

        // Feed the outline segments into rasterizer. These are later queried
        // for scanline crossings and minimum distance from a point to the outline.
        let mut rasterizer = Rasterizer::new();
        let mut mindist = OutlineDistance::new();
        for contour in outline.contours_iter() {
            let mut p0 = vec2_from_ft(contour.start(), unit_size);
            for curve in contour {
                match curve {
                    ft::outline::Curve::Line(a) => {
                        let p1 = vec2_from_ft(a, unit_size);
                        rasterizer.push_line(p0, p1);
                        mindist.push_line(p0, p1);
                        p0 = p1;
                    }
                    ft::outline::Curve::Bezier2(a, b) => {
                        let p1 = vec2_from_ft(a, unit_size);
                        let p2 = vec2_from_ft(b, unit_size);
                        rasterizer.push_bezier2(p0, p1, p2);
                        mindist.push_bezier2(p0, p1, p2);
                        p0 = p2;
                    }
                    ft::outline::Curve::Bezier3(a, b, c) => {
                        let p1 = vec2_from_ft(a, unit_size);
                        let p2 = vec2_from_ft(b, unit_size);
                        let p3 = vec2_from_ft(c, unit_size);
                        rasterizer.push_bezier3(p0, p1, p2, p3);
                        mindist.push_bezier3(p0, p1, p2, p3);
                        p0 = p3;
                    }
                }
            }
        }

        // Render
        for yr in 0 .. self.height {
            let buffer_offset = (self.y + yr) * pitch + self.x;
            let buffer_row = &mut buffer[buffer_offset .. buffer_offset + self.width];

            let y = (self.ymin + (self.height - yr - 1) as isize) as f32 + 0.5;

            let ref mut crossings = rasterizer.scanline_crossings(y);

            // Find point distance
            let mut crossings_idx = 0;
            let mut wn = 0i32;
            for xr in 0 .. self.width {
                let x = (self.xmin + xr as isize) as f32 + 0.5;
                let mp = Vec2::new(x, y);

                // Compute the distance
                let mut dist_min = mindist.distance(mp);

                // Is the point inside curve?
                while crossings.len() > crossings_idx && crossings[crossings_idx].x <= x {
                    wn += crossings[crossings_idx].dir as i32;
                    crossings_idx += 1;
                }
                let inside = if reverse_fill { wn < 0 } else { wn > 0 };
                if inside {
                    dist_min = -dist_min;
                }

                // Convert float distance to discrete space (u8):
                // 0 << 127 = outside
                // 127 = zero distance (the outline)
                // 128 >> 255 = inside
                let shift = 127.0;
                let scale = 1920. / face_size as f32;
                dist_min = shift - dist_min * scale;
                if dist_min < 0. { dist_min = 0.; }
                if dist_min > 255. { dist_min = 255.; }
                buffer_row[xr] = dist_min as u8;
            }
        }
    }
}

pub struct Font {
    // font texture buffer and size
    pub buffer: Vec<u8>,
    pub width: usize,
    pub height: usize,
    // metrics for glyphs contained in the texture
    pub glyphs: HashMap<char, Glyph>,
}

impl Font {
    pub fn new(square_size: usize) -> Self {
        Font {
            buffer: Vec::with_capacity(square_size * square_size),
            width: square_size,
            height: square_size,
            glyphs: HashMap::new(),
        }
    }

    pub fn build_from_file<P>(&mut self, path: P, face_index: isize, face_size: usize, padding: usize, chars: &str)
        where P: AsRef<path::Path>
    {
        let library = ft::Library::init().unwrap();
        let face = library.new_face(path.as_ref(), face_index).unwrap();
        self.build_from_face(&face, face_size, padding, chars)
    }

    pub fn build_from_face(&mut self, face: &ft::Face, face_size: usize, padding: usize, chars: &str) {
        let packer_config = rect_packer::Config {
            width: self.width as i32,
            height: self.height as i32,
            border_padding: 0,
            rectangle_padding: 0,
        };
        let mut packer = rect_packer::Packer::new(packer_config);

        self.glyphs.reserve(chars.len());
        self.buffer.resize(self.width * self.height, 0u8);

        face.set_pixel_sizes(face.em_size() as u32, 0).unwrap();

        for ch in chars.chars() {
            face.load_char(ch as usize, ft::face::NO_HINTING).unwrap();
            let mut glyph = Glyph::from_face(&face, face_size, padding);

            if let Some(rect) = packer.pack(glyph.width as i32, glyph.height as i32, false) {
                glyph.x = rect.x as usize;
                glyph.y = rect.y as usize;
            } else {
                panic!("font texture not large enough");
            }

            glyph.render_sdf(&face, face_size, &mut self.buffer, self.width);

            //println!("{} {:#?}", ch, glyph);
            self.glyphs.insert(ch, glyph);
        }
    }
}
