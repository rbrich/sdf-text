extern crate cgmath;
extern crate roots;

use std::f32;
use cgmath::prelude::*;
use cgmath::{Point2, Vector2};

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;


// Distance to a line segment from a point
// ---------------------------------------

pub fn line_distance(p: Point, p0: Point, p1: Point) -> f32 {
    let m = p - p0;
    let a = p1 - p0;
    let t = (m.dot(a) / a.dot(a))
            .max(0.0).min(1.0);
    let x = p0 + t * a;
    let side = m.perp_dot(a);
    let dist = (x - p).magnitude();
    if side < 0.0 {
        -dist
    } else {
        dist
    }
}


// Distance to quadratic bézier curve from a point
// -----------------------------------------------

fn quadratic_bezier(t: f32, p0: Point, p1: Point, p2: Point) -> Point {
    let tc = 1.0 - t;
    Point2::from_vec((tc*tc*p0).to_vec() + (2.0*tc*t*p1).to_vec() + (t*t*p2).to_vec())
}

fn quadratic_derivate(t: f32, p0: Point, p1: Point, p2: Point) -> Vector {
    let tc = 1.0 - t;
    2.0*tc*(p1 - p0) + 2.0*t*(p2 - p1)
}

pub fn quadratic_distance(p: Point, p0: Point, p1: Point, p2: Point) -> f32 {
    let m = p0 - p;
    let a = p1 - p0;
    let b = p2 - p1 - a;
    // Cubic equation coefficients
    let a3 = b.dot(b);
    let a2 = 3.0*a.dot(b);
    let a1 = 2.0*a.dot(a) + m.dot(b);
    let a0 = m.dot(a);
    // Find roots of the equation (1 or 3 real roots)
    let mut candidate_t = Vec::<f32>::with_capacity(5);
    match roots::find_roots_cubic(a3, a2, a1, a0) {
        roots::Roots::One(t) => candidate_t.extend_from_slice(&t),
        roots::Roots::Three(t) => candidate_t.extend_from_slice(&t),
        _ => unreachable!(),
    }
    // Drop roots outside of curve interval
    candidate_t.retain(|&t| t >= 0.0 && t <= 1.0);
    // Compute point on the curve for each t
    let mut candidate_x = Vec::<Point>::with_capacity(5);
    for &t in &candidate_t {
        candidate_x.push(quadratic_bezier(t, p0, p1, p2));
    }
    // Add end points
    candidate_t.push(0.0); candidate_x.push(p0);
    candidate_t.push(1.0); candidate_x.push(p2);
    // Find least distance point from candidates
    let mut dist_min = f32::INFINITY;
    let mut x_point = Point::new(0.0, 0.0);
    let mut x_t = 0f32;
    for (t, x) in candidate_t.into_iter().zip(candidate_x.into_iter()) {
        // Actually, it's distance squared, but that's okay for comparison
        let dist = (x - p).magnitude2();
        if dist < dist_min {
            dist_min = dist;
            x_point = x;
            x_t = t;
        }
    }
    dist_min = dist_min.sqrt();
    // Determine sign (curve side)
    let direction = quadratic_derivate(x_t, p0, p1, p2);
    let side = (p - x_point).perp_dot(direction);
    if side < 0.0 {
        -dist_min
    } else {
        dist_min
    }
}


// Distance to cubic bézier curve from a point
// -------------------------------------------

fn cubic_bezier(t: f32, p0: Point, p1: Point, p2: Point, p3: Point) -> Point {
    let tc = 1.0 - t;
    Point2::from_vec((tc*tc*tc*p0).to_vec() +
                     (3.0*tc*tc*t*p1).to_vec() +
                     (3.0*tc*t*t*p2).to_vec() +
                     (t*t*t*p3).to_vec())
}

fn cubic_derivate(t: f32, p0: Point, p1: Point, p2: Point, p3: Point) -> Vector {
    let tc = 1.0 - t;
    3.0*tc*tc*(p1 - p0) + 6.0*tc*t*(p2 - p1) + 3.0*t*t*(p3 - p2)
}

pub fn cubic_distance(p: Point, p0: Point, p1: Point, p2: Point, p3: Point) -> f32 {
    let f = |t| { (cubic_bezier(t, p0, p1, p2, p3) - p).dot(cubic_derivate(t, p0, p1, p2, p3)) };
    // Find roots of the equation (up to 5 real roots)
    let mut candidate_t = Vec::<f32>::with_capacity(7);
    let convergency = roots::SimpleConvergency { eps:2e-5f32, max_iter:100 };
    let steps = 15;
    let mut a = 0.0;
    for t in 1 .. steps + 1 {
        let b = t as f32 / steps as f32;
        match roots::find_root_brent(a, b, &f, &convergency) {
            Ok(t) => candidate_t.push(t),
            Err(_) => (),
        }
        a = b;
    }
    // Compute point on the curve for each t
    let mut candidate_x = Vec::<Point>::with_capacity(7);
    for &t in &candidate_t {
        candidate_x.push(cubic_bezier(t, p0, p1, p2, p3));
    }
    // Add end points
    candidate_t.push(0.0); candidate_x.push(p0);
    candidate_t.push(1.0); candidate_x.push(p3);
    // Find least distance point from candidates
    let mut dist_min = f32::INFINITY;
    let mut x_point = Point::new(0.0, 0.0);
    let mut x_t = 0f32;
    for (t, x) in candidate_t.into_iter().zip(candidate_x.into_iter()) {
        // Actually, it's distance squared, but that's okay for the comparison
        let dist = (x - p).magnitude2();
        if dist < dist_min {
            dist_min = dist;
            x_point = x;
            x_t = t;
        }
    }
    dist_min = dist_min.sqrt();
    // Determine sign (curve side)
    let direction = cubic_derivate(x_t, p0, p1, p2, p3);
    let side = (p - x_point).perp_dot(direction);
    if side < 0.0 {
        -dist_min
    } else {
        dist_min
    }
}


// Tests
// -----

#[cfg(test)]
mod tests {
    use super::*;

    fn float_eq(a: f32, b: f32) -> bool {
        let eps = 4e-5f32;
        let d = (a - b).abs();
        println!("float_eq({}, {}): d={}, eps={}", a, b, d, eps);
        d < eps
    }

    /*
    Test samples (from distance.py):

    Curve level: 4
    Curve points: [(100.0, 200.0), (250.0, 400.0), (400.0, 200.0), (400.0, 400.0)]
    Query point: (98.0, 314.0)
    Result: dist=-80.05094469021948, X=[ 148.925869    252.23666449], t=0.1091577060749022

    Curve level: 4
    Curve points: [(100.0, 200.0), (250.0, 400.0), (400.0, 200.0), (400.0, 400.0)]
    Query point: (419.0, 291.0)
    Result: dist=47.04632869336913, X=[ 382.2548382   320.37941673], t=0.7942392383680202
    */
    #[test]
    fn test_cubic_distance() {
        let p = Point::new(98.0, 314.0);
        let p0 = Point::new(100.0, 200.0);
        let p1 = Point::new(250.0, 400.0);
        let p2 = Point::new(400.0, 200.0);
        let p3 = Point::new(400.0, 400.0);
        let dist = cubic_distance(p, p0, p1, p2, p3);
        assert!(float_eq(dist, -80.05094469021948));

        let p = Point::new(419.0, 291.0);
        let dist = cubic_distance(p, p0, p1, p2, p3);
        assert!(float_eq(dist, 47.04632869336913));
        /*
        let (a, b) = (0.73333335, 0.8);
        let f = |t| { (cubic_bezier(t, p0, p1, p2, p3) - p).dot(cubic_derivate(t, p0, p1, p2, p3)) };
        println!("f(a)={}, f(b)={}", f(a), f(b));

        let convergency = roots::SimpleConvergency { eps:2e-5f32, max_iter:100 };
        let res = roots::find_root_brent(a, b, &f, &convergency);
        println!("brent: {:?}", res);

        let t = 0.7942392383680202;
        println!("t={}, f(t)={}", t, f(t));
        */
    }
}
