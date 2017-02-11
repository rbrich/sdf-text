use curve::*;

#[derive(Copy, Clone, Debug)]
pub struct OrientedCrossing {
    pub dir: i8,  // 1=up, -1=down
    pub x: f32,
}

impl OrientedCrossing {
    pub fn new(dir: i8, x: f32) -> Self {
        OrientedCrossing { dir: dir, x: x }
    }
}

#[derive(Clone, Debug)]
pub struct Spans {
    pub crossings: Vec<OrientedCrossing>,
}

impl Spans {
    pub fn new() -> Self {
        Spans {
            crossings: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rasterizer {
    pub scanlines: Vec<Spans>,  // from bottom to top
    pub origin_y: f32,          // y coordinate of first scan line
}

impl Rasterizer {
    pub fn new(rows: usize, origin_y: f32) -> Self {
        let mut res = Rasterizer {
            scanlines: Vec::with_capacity(rows),
            origin_y: origin_y,
        };
        res.scanlines.resize(rows, Spans::new());
        res
    }

    pub fn push_line(&mut self, p0: Vec2, p1: Vec2) {
        if p0.y < p1.y {
            // up
            self.push_line_unoriented(p0, p1, 1);
        }
        if p1.y < p0.y {
            // down
            self.push_line_unoriented(p1, p0, -1);
        }
    }

    // Pre-condition: p0.y < p1.y
    fn push_line_unoriented(&mut self, p0: Vec2, p1: Vec2, dir: i8) {
        let mut line = (p0.y - self.origin_y) as i32 - 1;
        let mut y = line as f32 + self.origin_y;
        while y < p0.y {
            y += 1.0;
            line += 1;
        }
        let mut x = line_intersection(y, p0, p1);
        let dx = line_step(p0, p1);
        while y < p1.y {
            self.scanlines[line as usize].crossings.push(OrientedCrossing::new(dir, x));
            x += dx;
            y += 1.0;
            line += 1;
        }
    }

    pub fn push_bezier2(&mut self, p0: Vec2, p1: Vec2, p2: Vec2) {
        let eps = 0.1;
        let subpixel = (p0 - p1 + p2 - p1).magnitude2() < 0.5;
        if p0.y < p2.y && (subpixel || collinear(p0, p1, p2, eps)) {
            // up line
            return self.push_line_unoriented(p0, p2, 1);
        }
        if p2.y < p0.y && (subpixel || collinear(p2, p1, p0, eps)) {
            // down line
            return self.push_line_unoriented(p2, p0, -1);
        }
        // not flat enough -> split
        let t = 0.5;
        let m0 = p0.lerp(p1, t);
        let m1 = p1.lerp(p2, t);
        let m = m0.lerp(m1, t);
        self.push_bezier2(p0, m0, m);
        self.push_bezier2(m, m1, p2);
    }

    pub fn push_bezier3(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) {
        /*if y >= min(&[p0.y, p1.y, p2.y, p3.y])
        && y < max(&[p0.y, p1.y, p2.y, p3.y]) {
            let (x_num, x_arr) = cubic_intersection(y, p0, p1, p2, p3);
            for i in 0..x_num {
                intersections.push(x_arr[i]);
            }
            // Include bottom point, if touched
            if x_num == 0 && y == p0.y && p0.y < p3.y {
                intersections.push(Intersection::new(true, p0.x));
            }
            if x_num == 0 && y == p3.y && p3.y < p0.y {
                intersections.push(Intersection::new(false, p0.x));
            }
        }*/
    }
}
