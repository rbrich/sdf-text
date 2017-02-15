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
pub struct LinearProfile {
    dir: i8,
    p0: Vec2,
    p1: Vec2,
}

impl LinearProfile {
    pub fn new(dir: i8, p0: Vec2, p1: Vec2) -> Self {
        LinearProfile {
            dir: dir,
            p0: p0,
            p1: p1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuadraticProfile {
    dir: i8,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
}

impl QuadraticProfile {
    pub fn new(dir: i8, p0: Vec2, p1: Vec2, p2: Vec2) -> Self {
        QuadraticProfile {
            dir: dir,
            p0: p0,
            p1: p1,
            p2: p2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rasterizer {
    pub linear_profiles: Vec<LinearProfile>,
    pub quadratic_profiles: Vec<QuadraticProfile>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Rasterizer {
            linear_profiles: Vec::new(),
            quadratic_profiles: Vec::new(),
        }
    }

    pub fn scanline_crossings(&self, y: f32) -> Vec<OrientedCrossing> {
        let mut crossings = Vec::<OrientedCrossing>::new();
        for prf in &self.linear_profiles {
            if y >= prf.p0.y && y < prf.p1.y {
                let x = line_intersection(y, prf.p0, prf.p1);
                crossings.push(OrientedCrossing::new(prf.dir, x));
            }
        }
        for prf in &self.quadratic_profiles {
            if y >= prf.p0.y && y < prf.p2.y {
                let (x_n, x_a) = quadratic_intersection(y, prf.p0, prf.p1, prf.p2);
                assert!(x_n == 1);
                crossings.push(OrientedCrossing::new(prf.dir, x_a[0]));
            }
        }
        crossings.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        //println!("{} {:?}", y, crossings);
        crossings
    }

    pub fn push_line(&mut self, p0: Vec2, p1: Vec2) {
        if p0.y < p1.y {
            self.linear_profiles.push(LinearProfile::new(1, p0, p1));
        }
        if p1.y < p0.y {
            self.linear_profiles.push(LinearProfile::new(-1, p1, p0));
        }
    }

    pub fn push_bezier2(&mut self, p0: Vec2, p1: Vec2, p2: Vec2) {
        // check the parabola for Y extrema
        let t = (p0.y - p1.y) / (p0.y - 2.0 * p1.y + p2.y);
        if t.is_finite() && 0.0 < t && t < 1.0 {
            // extreme point found, split the curve at `t`
            let m0 = p0.lerp(p1, t);
            let m1 = p1.lerp(p2, t);
            let m = m0.lerp(m1, t);
            self.push_bezier2_monotonic(p0, m0, m);
            self.push_bezier2_monotonic(m, m1, p2);
        } else {
            self.push_bezier2_monotonic(p0, p1, p2);
        }
    }

    pub fn push_bezier2_monotonic(&mut self, p0: Vec2, p1: Vec2, p2: Vec2) {
        if p0.y < p2.y {
            self.quadratic_profiles.push(QuadraticProfile::new(1, p0, p1, p2));
        }
        if p2.y < p0.y {
            self.quadratic_profiles.push(QuadraticProfile::new(-1, p2, p1, p0));
        }
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
