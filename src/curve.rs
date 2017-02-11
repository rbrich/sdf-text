use std;
use std::f32;
use roots;


#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x: x, y: y}
    }
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn magnitude2(self) -> f32 {
        self.dot(self)
    }
    pub fn magnitude(self) -> f32 {
        f32::sqrt(self.magnitude2())
    }
    // linear interpolation
    pub fn lerp(self, other: Vec2, t: f32) -> Vec2 {
        (1.0 - t) * self + t * other
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self * rhs.x, y: self * rhs.y }
    }
}


// Line segment
// ------------
// B(t) = p0 + t * (p1 - p0); t = 0..1

// Distance to a line segment from a point
pub fn line_distance(p: Vec2, p0: Vec2, p1: Vec2) -> f32 {
    let m = p - p0;
    let a = p1 - p0;
    let t = (m.dot(a) / a.dot(a))
            .max(0.0).min(1.0);
    let x = p0 + t * a;
    (x - p).magnitude()
}

// Intersection between horizontal line at Y and line segment
pub fn line_intersection(y: f32, p0: Vec2, p1: Vec2) -> f32 {
    let t = (y - p0.y) / (p1.y - p0.y);
    p0.x + t * (p1.x - p0.x)
}

// Pre-condition: p0.y < p1.y
#[inline]
pub fn line_step(p0: Vec2, p1: Vec2) -> f32 {
    (p1.x - p0.x) / (p1.y - p0.y)
}

pub fn collinear(p0: Vec2, p1: Vec2, p2: Vec2, eps: f32) -> bool {
    let a = line_step(p0, p1);
    let b = line_step(p0, p2);
    if a.is_finite() && b.is_finite() {
        (a - b).abs() < eps
    } else {
        false
    }
}


// Quadratic (conic) segment
// -------------------------
// B(t) = (1-t)^2 * p0 + 2*(1-t)*t * p1 + t^2 * p2; t = 0..1

fn quadratic_bezier(t: f32, p0: Vec2, p1: Vec2, p2: Vec2) -> Vec2 {
    let tc = 1.0 - t;
    tc*tc*p0 + 2.0*tc*t*p1 + t*t*p2
}

fn quadratic_derivate(t: f32, p0: Vec2, p1: Vec2, p2: Vec2) -> Vec2 {
    let tc = 1.0 - t;
    2.0*tc*(p1 - p0) + 2.0*t*(p2 - p1)
}

// Distance to quadratic bézier curve from a point
pub fn quadratic_distance(p: Vec2, p0: Vec2, p1: Vec2, p2: Vec2) -> f32 {
    let m = p0 - p;
    let a = p1 - p0;
    let b = p2 - p1 - a;
    // Cubic equation coefficients
    let a3 = b.dot(b);
    let a2 = 3.0*a.dot(b);
    let a1 = 2.0*a.dot(a) + m.dot(b);
    let a0 = m.dot(a);
    // Find roots of the equation (1 or 3 real roots)
    let mut candidates = Vec::<Vec2>::with_capacity(5);
    for &t in roots::find_roots_cubic(a3, a2, a1, a0).as_ref() {
        // Drop roots outside of curve interval
        if t >= 0.0 && t <= 1.0 {
            // Compute point on the curve for each t
            candidates.push(quadratic_bezier(t, p0, p1, p2));
        }
    }
    // Add end points
    candidates.push(p0);
    candidates.push(p2);
    // Find least distance point from candidates
    let mut dist_min = f32::INFINITY;
    for x in candidates.into_iter() {
        // Actually, it's distance squared, but that's okay for comparison
        let dist = (x - p).magnitude2();
        if dist < dist_min {
            dist_min = dist;
        }
    }
    dist_min.sqrt()
}
/*
pub fn quadratic_intersection(y: f32, p0: Vec2, p1: Vec2, p2: Vec2) -> (usize, [;3]) {
    let a2 = p0.y - 2.0*p1.y + p2.y;
    let a1 = -2.0*p0.y + 2.0*p1.y;
    let a0 = p0.y - y;
    let mut x_arr = [Intersection::new(true, 0.); 3];
    let mut x_num = 0usize;
    for &t in roots::find_roots_quadratic(a2, a1, a0).as_ref() {
        if t < 0.0 || t > 1.0 {
            continue;
        }
        let tc = 1.0 - t;
        x_arr[x_num].x = tc*tc * p0.x + 2.0*tc*t * p1.x + t*t * p2.x;
        let direction = quadratic_derivate(t, p0, p1, p2);
        x_arr[x_num].up = direction.y > 0.0;
        x_num += 1;
    }
    (x_num, x_arr)
}
*/

// Cubic segment
// -------------
// B(t) = (1-t)^3*p0 + 3*(1-t)^2*t*p1 + 3*(1-t)*t^2*p2 + t^3*p3; t = 0..1

fn cubic_bezier(t: f32, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
    let tc = 1.0 - t;
    tc*tc*tc*p0 + 3.0*tc*tc*t*p1 + 3.0*tc*t*t*p2 + t*t*t*p3
}

fn cubic_derivate(t: f32, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
    let tc = 1.0 - t;
    3.0*tc*tc*(p1 - p0) + 6.0*tc*t*(p2 - p1) + 3.0*t*t*(p3 - p2)
}

// Distance to quadratic bézier curve from a point
pub fn cubic_distance(p: Vec2, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let f = |t| { (cubic_bezier(t, p0, p1, p2, p3) - p).dot(cubic_derivate(t, p0, p1, p2, p3)) };
    // Find roots of the equation (up to 5 real roots)
    let mut candidates = Vec::<Vec2>::with_capacity(7);
    let convergency = roots::SimpleConvergency { eps:2e-5f32, max_iter:100 };
    let steps = 15;
    let mut a = 0.0;
    for t in 1 .. steps + 1 {
        let b = t as f32 / steps as f32;
        match roots::find_root_brent(a, b, &f, &convergency) {
            // Compute point on the curve for each t
            Ok(t) => candidates.push(cubic_bezier(t, p0, p1, p2, p3)),
            Err(_) => (),
        }
        a = b;
    }
    // Add end points
    candidates.push(p0);
    candidates.push(p3);
    // Find least distance point from candidates
    let mut dist_min = f32::INFINITY;
    for x in candidates.into_iter() {
        // Actually, it's distance squared, but that's okay for the comparison
        let dist = (x - p).magnitude2();
        if dist < dist_min {
            dist_min = dist;
        }
    }
    dist_min.sqrt()
}

/*
pub fn cubic_intersection(y: f32, p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> (usize, [Intersection;3]) {
    let a3 = -p0.y + 3.0*p1.y -3.0*p2.y + p3.y;
    let a2 = 3.0*p0.y - 6.0*p1.y + 3.0*p2.y;
    let a1 = -3.0*p0.y + 3.0*p1.y;
    let a0 = p0.y - y;
    let mut x_arr = [Intersection::new(true, 0.); 3];
    let mut x_num = 0usize;
    for &t in roots::find_roots_cubic(a3, a2, a1, a0).as_ref() {
        if !t.is_finite() || t < 0.0 || t > 1.0 {
            continue;
        }
        let tc = 1.0 - t;
        let x = tc*tc*tc * p0.x + 3.0*tc*tc*t * p1.x + 3.0*tc*t*t * p2.x + t*t*t * p3.x;
        let direction = cubic_derivate(t, p0, p1, p2, p3);
        x_arr[x_num].x = x;
        x_arr[x_num].up = direction.y > 0.0;
        x_num += 1;
    }
    (x_num, x_arr)
}
*/

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
