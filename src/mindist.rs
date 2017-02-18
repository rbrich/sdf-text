use curve::*;
use std::f32;

#[derive(Clone, Debug)]
pub struct OutlineDistance {
    pub linear_segments: Vec<LinearSegment>,
    pub quadratic_segments: Vec<QuadraticSegment>,
    pub cubic_segments: Vec<CubicSegment>,
}

impl OutlineDistance {
    pub fn new() -> Self {
        OutlineDistance {
            linear_segments: Vec::new(),
            quadratic_segments: Vec::new(),
            cubic_segments: Vec::new(),
        }
    }

    pub fn push_line(&mut self, p0: Vec2, p1: Vec2) {
        self.linear_segments.push(LinearSegment::new(p0, p1));
    }

    pub fn push_bezier2(&mut self, p0: Vec2, p1: Vec2, p2: Vec2) {
        self.quadratic_segments.push(QuadraticSegment::new(p0, p1, p2));
    }

    pub fn push_bezier3(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) {
        self.cubic_segments.push(CubicSegment::new(p0, p1, p2, p3));
    }

    pub fn distance(&self, p: Vec2) -> f32 {
        let mut dist_min = f32::INFINITY;
        for sgt in &self.linear_segments {
            let dist = sgt.distance(p);
            if dist < dist_min {
                dist_min = dist;
            }
        }
        for sgt in &self.quadratic_segments {
            let dist = sgt.distance(p);
            if dist < dist_min {
                dist_min = dist;
            }
        }
        for sgt in &self.cubic_segments {
            let dist = sgt.distance(p);
            if dist < dist_min {
                dist_min = dist;
            }
        }
        dist_min
    }
}
