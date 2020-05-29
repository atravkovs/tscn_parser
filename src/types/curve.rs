use nalgebra::Point2;

type Point = Point2<f32>;

#[derive(Debug, Clone, Copy)]
pub struct ControlPoint {
    pub pos: Point,
    pub left_tangent: f32,
    pub right_tangent: f32,
}

impl ControlPoint {
    pub fn new(pos: Point, left_tangent: f32, right_tangent: f32) -> Self {
        ControlPoint {
            pos,
            left_tangent,
            right_tangent,
        }
    }

    pub fn new_point(x: f32, y: f32, left_tangent: f32, right_tangent: f32) -> Self {
        Self::new(Point::new(x, y), left_tangent, right_tangent)
    }
}

#[derive(Debug, Clone)]
pub struct Curve {
    points: Vec<ControlPoint>,
}

impl Default for Curve {
    fn default() -> Self {
        Curve { points: Vec::new() }
    }
}

impl Curve {
    const CMP_EPSILON: f32 = 0.00001;

    /// Calculates bezier Interpolation
    ///
    /// According to formula from Wikipedia: https://en.wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves
    fn bezier_interp(t: f32, start: f32, control1: f32, control2: f32, end: f32) -> f32 {
        let omt = 1.0 - t;
        let omt2 = omt * omt;
        let omt3 = omt2 * omt;
        let t2 = t * t;
        let t3 = t2 * t;

        start * omt3 + control1 * omt2 * t * 3.0 + control2 * omt * t2 * 3.0 + end * t3
    }

    pub fn add_point(&mut self, control_point: ControlPoint) {
        self.points.push(control_point);
    }

    pub fn get_index(&self, offset: f32) -> usize {
        let mut imin = 0;
        let mut imax = self.points.len() - 1;

        while imax - imin > 1 {
            let m = (imin + imax) / 2;

            let a = self.points[m].pos.x;
            let b = self.points[m + 1].pos.x;

            if a < offset && b < offset {
                imin = m;
            } else if a > offset {
                imax = m;
            } else {
                return m;
            }
        }

        if offset > self.points[imax].pos.x {
            return imax;
        }

        imin
    }

    pub fn interpolate(&self, offset: f32) -> f32 {
        if self.points.len() == 0 {
            return 0.0;
        }
        if self.points.len() == 1 {
            return self.points[0].pos.y;
        }

        let i = self.get_index(offset);

        if i == self.points.len() - 1 {
            return self.points[i].pos.y;
        }

        let local = offset - self.points[i].pos.x;

        if i == 0 && local <= 0.0 {
            return self.points[0].pos.y;
        }

        self.interpolate_local(i, local)
    }

    fn interpolate_local(&self, index: usize, offset: f32) -> f32 {
        let point1 = self.points[index].pos;
        let point2 = self.points[index + 1].pos;
        let right_tangent = self.points[index].right_tangent;
        let left_tangent = self.points[index + 1].left_tangent;

        let mut d = point2.x - point1.x;
        if d.abs() <= Self::CMP_EPSILON {
            return point2.y;
        }

        let local_offset = offset / d;
        d /= 3.0;

        let yac = point1.y + d * right_tangent;
        let ybc = point2.y - d * left_tangent;

        Self::bezier_interp(local_offset, point1.y, yac, ybc, point2.y)
    }
}
