use roots;
use curve::*;

#[derive(Copy, Clone, Debug)]
pub struct OrientedCrossing {
    pub dir: i8,  // 1=left (line going up), -1=right (line going down)
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
pub struct CubicProfile {
    dir: i8,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
}

impl CubicProfile {
    pub fn new(dir: i8, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        CubicProfile {
            dir: dir,
            p0: p0,
            p1: p1,
            p2: p2,
            p3: p3,
        }
    }
}

/**
 * Generic rasterizer for vector graphics.
 *
 * Push curves in, evaluate scanline, read crossings.
 * Each crossing has X coordinate (crossed at scanline's Y coordinate)
 * and direction (curve crossed from left/right).
 * This gives us enough information to compute winding numbers (wn)
 * and fill the spans according to filling rule.
 *
 * For use with SDF, this has to give *exact* coordinates.
 * Any estimation will create visible artifacts, eg. if the distance vector
 * points inside while rasterizer evaluates the point as outside, we get
 * a blot at the place.
 */
#[derive(Clone, Debug)]
pub struct Rasterizer {
    pub linear_profiles: Vec<LinearProfile>,
    pub quadratic_profiles: Vec<QuadraticProfile>,
    pub cubic_profiles: Vec<CubicProfile>,
}

impl Rasterizer {
    pub fn new() -> Self {
        Rasterizer {
            linear_profiles: Vec::new(),
            quadratic_profiles: Vec::new(),
            cubic_profiles: Vec::new(),
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
                let x = quadratic_intersection(y, prf.p0, prf.p1, prf.p2);
                crossings.push(OrientedCrossing::new(prf.dir, x));
            }
        }
        for prf in &self.cubic_profiles {
            if y >= prf.p0.y && y < prf.p3.y {
                let x = cubic_intersection(y, prf.p0, prf.p1, prf.p2, prf.p3);
                crossings.push(OrientedCrossing::new(prf.dir, x));
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
        // check the curve for Y extrema
        let t = (p0.y - p1.y) / (p0.y - 2.0 * p1.y + p2.y);
        if t.is_finite() && 0.0 < t && t < 1.0 {
            // one extremum found, split the curve at `t`
            let m0 = p0.lerp(p1, t);
            let m1 = p1.lerp(p2, t);
            let n0 = m0.lerp(m1, t);
            self.push_bezier2_monotonic(p0, m0, n0);
            self.push_bezier2_monotonic(n0, m1, p2);
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
        // check the curve for Y extrema
        let a = p3.y - 3.0*p2.y + 3.0*p1.y - p0.y;
        let b = 2.0*(p2.y - 2.0*p1.y + p0.y);
        let c = p1.y - p0.y;
        let found_roots = roots::find_roots_quadratic(a, b, c);
        let extrema: Vec<f32> = found_roots.as_ref().iter().cloned()
            .filter(|&t| t.is_finite() && 0.0 < t && t < 1.0 ).collect();
        if extrema.len() == 0 {
            // No extrema, the curve is monotonic
            self.push_bezier3_monotonic(p0, p1, p2, p3)
        } else {
            // one or more extrema found, split the curve at `t`
            let t1 = extrema[0];
            let m0 = p0.lerp(p1, t1);
            let m1 = p1.lerp(p2, t1);
            let m2 = p2.lerp(p3, t1);
            let n0 = m0.lerp(m1, t1);
            let n1 = m1.lerp(m2, t1);
            let o0 = n0.lerp(n1, t1);
            if extrema.len() > 1 {
                // If there is second extremum, split the curve recursively
                // (we could also do double split in one go as an optimization)
                debug_assert!(extrema.len() == 2);
                let t2 = extrema[1];
                if t2 > t1 {
                    self.push_bezier3_monotonic(p0, m0, n0, o0);
                    self.push_bezier3(o0, n1, m2, p3);
                } else {
                    self.push_bezier3(p0, m0, n0, o0);
                    self.push_bezier3_monotonic(o0, n1, m2, p3);
                }
            } else {
                self.push_bezier3_monotonic(p0, m0, n0, o0);
                self.push_bezier3_monotonic(o0, n1, m2, p3);
            }
        }
    }

    pub fn push_bezier3_monotonic(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) {
        if p0.y < p3.y {
            self.cubic_profiles.push(CubicProfile::new(1, p0, p1, p2, p3));
        }
        if p3.y < p0.y {
            self.cubic_profiles.push(CubicProfile::new(-1, p3, p2, p1, p0));
        }
    }
}
