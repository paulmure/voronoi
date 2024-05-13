use std::{
    cmp::min,
    fmt,
    ops::{Add, Mul, Sub},
};

use ordered_float::OrderedFloat;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: OrderedFloat<f64>,
    pub y: OrderedFloat<f64>,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x.into_inner(), self.y.into_inner())
    }
}

impl Point {
    pub fn new(x: OrderedFloat<f64>, y: OrderedFloat<f64>) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<&Point> for Point {
    type Output = Self;

    fn add(self, other: &Point) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<OrderedFloat<f64>> for Point {
    type Output = Self;

    fn mul(self, rhs: OrderedFloat<f64>) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<OrderedFloat<f64>> for &Point {
    type Output = Point;

    fn mul(self, rhs: OrderedFloat<f64>) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub type Segment = [Point; 2];

pub struct BoundingBox {
    x_min: OrderedFloat<f64>,
    x_max: OrderedFloat<f64>,
    y_min: OrderedFloat<f64>,
    y_max: OrderedFloat<f64>,
}

impl BoundingBox {
    pub fn new(
        x_min: OrderedFloat<f64>,
        x_max: OrderedFloat<f64>,
        y_min: OrderedFloat<f64>,
        y_max: OrderedFloat<f64>,
    ) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

pub fn point_on_arc_at_x(focus: &Point, yl: OrderedFloat<f64>, x: OrderedFloat<f64>) -> Point {
    let xf = focus.x;
    let yf = focus.y;

    let dx = x - xf;
    let dx2 = dx * dx;
    let dy = yf - yl;

    if dy == 0.0 {
        Point::new((focus.x + x) / 2.0, yl)
    } else {
        Point::new(x, dx2 / (dy * 2.0) + (yf + yl) / 2.0)
    }

    // let y = if dy == 0.0 {
    //     yl
    // } else {
    //     dx2 / (dy * 2.0) + (yf + yl) / 2.0
    // };

    // Point::new(x, y)
}

pub fn normal_vector(point: Point) -> Point {
    Point::new(-point.y, point.x)
}

pub fn circumcenter(a: &Point, b: &Point, c: &Point) -> Point {
    let x1 = a.x;
    let y1 = a.y;
    let x2 = b.x;
    let y2 = b.y;
    let x3 = c.x;
    let y3 = c.y;

    let c1 = x3 * x3 + y3 * y3 - x1 * x1 - y1 * y1;
    let c2 = x3 * x3 + y3 * y3 - x2 * x2 - y2 * y2;
    let a1 = (x1 - x3) * -2.;
    let a2 = (x2 - x3) * -2.;
    let b1 = (y1 - y3) * -2.;
    let b2 = (y2 - y3) * -2.;

    let numer = c1 * a2 - c2 * a1;
    let denom = b1 * a2 - b2 * a1;

    if denom == 0.0 {
        panic!("circle center does not exist");
    }
    let y_cen = numer / denom;

    let x_cen = if a2 != 0.0 {
        (c2 - b2 * y_cen) / a2
    } else {
        (c1 - b1 * y_cen) / a1
    };

    Point::new(x_cen, y_cen)
}

pub fn intersection(ao: &Point, ad: &Point, bo: &Point, bd: &Point) -> Option<Point> {
    let dx = bo.x - ao.x;
    let dy = bo.y - ao.y;
    let det = bd.x * ad.y - bd.y * ad.x;
    if det == OrderedFloat(0.0) {
        return None;
    }

    let u = (dy * bd.x - dx * bd.y) / det;
    let v = (dy * ad.x - dx * ad.y) / det;
    if u.signum() != v.signum() {
        return None;
    }

    Some((ad * u) + ao)
}

pub fn distance(a: &Point, b: &Point) -> OrderedFloat<f64> {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    OrderedFloat((dx * dx + dy * dy).sqrt())
}

pub fn breakpoint_at_x(l: &Point, r: &Point, yl: OrderedFloat<f64>) -> OrderedFloat<f64> {
    let ax = l.x;
    let bx = r.x;
    let ay = l.y;
    let by = r.y;

    // shift frames
    let bx_s = bx - ax;
    let ay_s = ay - yl;
    let by_s = by - yl;

    let discrim = ay_s * by_s * ((ay_s - by_s) * (ay_s - by_s) + bx_s * bx_s);
    let numer = ay_s * bx_s - discrim.sqrt();
    let denom = ay_s - by_s;

    let mut x_bp = if denom != 0.0 {
        numer / denom
    } else {
        bx_s / 2.
    };
    x_bp += ax; // shift back to original frame

    x_bp
}

pub fn bounded_segment(origin: &Point, direction: &Point, bounding_box: &BoundingBox) -> Segment {
    let x_min = bounding_box.x_min;
    let x_max = bounding_box.x_max;
    let y_min = bounding_box.y_min;
    let y_max = bounding_box.y_max;

    let x = origin.x;
    let y = origin.y;
    let dx = direction.x;
    let dy = direction.y;

    let cx: OrderedFloat<f64> = if dx == OrderedFloat(0.0) {
        0.0.into()
    } else if dx < OrderedFloat(0.0) {
        (x_min - x) / dx
    } else {
        (x_max - x) / dx
    };

    let cy: OrderedFloat<f64> = if dy == OrderedFloat(0.0) {
        0.0.into()
    } else if dy < OrderedFloat(0.0) {
        (y_min - y) / dy
    } else {
        (y_max - y) / dy
    };

    let c = if dx == OrderedFloat(0.0) {
        cy
    } else if dy == OrderedFloat(0.0) {
        cx
    } else {
        min(cx, cy)
    };
    let destination = Point {
        x: x + c * dx,
        y: y + c * dy,
    };

    [*origin, destination]
}

#[cfg(test)]
mod tests {
    use crate::test_utils::compare_segments;

    use super::*;

    #[test]
    fn bounding_box_vertical() {
        let bbox = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let origin = Point::new(500.0.into(), 500.0.into());
        let direction = Point::new(0.0.into(), 500.0.into());

        let gold = [
            Point::new(500.0.into(), 500.0.into()),
            Point::new(500.0.into(), 1000.0.into()),
        ];

        let seg = bounded_segment(&origin, &direction, &bbox);

        assert!(compare_segments(&gold, &seg));
    }

    #[test]
    fn bounding_box_vertical_neg_0() {
        let bbox = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let origin = Point::new(500.0.into(), 500.0.into());
        let direction = Point::new((-0.0).into(), 500.0.into());

        let gold = [
            Point::new(500.0.into(), 500.0.into()),
            Point::new(500.0.into(), 1000.0.into()),
        ];

        let seg = bounded_segment(&origin, &direction, &bbox);

        assert!(compare_segments(&gold, &seg));
    }

    #[test]
    fn bounding_box_vertical_neg_both() {
        let bbox = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let origin = Point::new(750.0.into(), 500.0.into());
        let direction = Point::new((-0.0).into(), (-500.0).into());

        let gold = [
            Point::new(750.0.into(), 500.0.into()),
            Point::new(750.0.into(), 0.0.into()),
        ];

        let seg = bounded_segment(&origin, &direction, &bbox);

        assert!(compare_segments(&gold, &seg));
    }
}
