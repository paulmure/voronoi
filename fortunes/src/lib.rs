use geometry::*;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::beachline::{Arc, Beachline, BreakPoint};

mod beachline;
pub mod geometry;
#[cfg(test)]
mod test_utils;

#[derive(Clone, Hash, PartialEq, Eq)]
enum Event {
    Site(Point),
    /// Index of arc associated with circle event
    Circle(usize),
}

type EventQueue = PriorityQueue<Event, OrderedFloat<f64>>;

pub fn fortunes_algorithm(sites: &Vec<Point>, bounding_box: &BoundingBox) -> Vec<Segment> {
    let mut eq = EventQueue::new();
    for site in sites {
        eq.push(Event::Site(*site), site.y);
    }

    let mut beachline = Beachline::new();
    let mut res = vec![];

    while let Some((e, yl)) = eq.pop() {
        match e {
            Event::Site(site) => add_parabola(site, yl, &mut eq, &mut beachline),
            Event::Circle(arc_idx) => {
                remove_parabola(arc_idx, &mut eq, &mut beachline, &mut res, yl)
            }
        }
    }

    beachline.extend_edges_to_bounding_box(bounding_box, &mut res);

    res
}

fn add_parabola(
    site: Point,
    yl: OrderedFloat<f64>,
    eq: &mut EventQueue,
    beachline: &mut Beachline,
) {
    if let Some((arc, arc_idx)) = beachline.arc_under_point(&site, yl) {
        remove_circle_event(arc_idx, eq);

        let a = Arc::new(arc.site);
        let b = Arc::new(site);
        let c = Arc::new(arc.site);

        let edge_origin = point_on_arc_at_x(&arc.site, yl, site.x);
        let xl = BreakPoint::new(edge_origin, normal_vector(a.site - b.site), a.site, b.site);
        let xr = BreakPoint::new(edge_origin, normal_vector(b.site - c.site), b.site, c.site);

        beachline.replace_arc(arc_idx, a, xl, b, xr, c, eq, yl);
    } else {
        beachline.add_first_parabola(site);
    }
}

fn remove_parabola(
    arc_idx: usize,
    eq: &mut EventQueue,
    beachline: &mut Beachline,
    segments: &mut Vec<Segment>,
    yl: OrderedFloat<f64>,
) {
    let p = beachline.arc(arc_idx);
    let (l, l_idx) = beachline.left_arc(arc_idx).unwrap();
    let (r, r_idx) = beachline.right_arc(arc_idx).unwrap();
    remove_circle_event(l_idx, eq);
    remove_circle_event(r_idx, eq);

    let s = circumcenter(&l.site, &p.site, &r.site);

    let (xl, xl_idx) = beachline.left_edge(arc_idx).unwrap();
    let (xr, xr_idx) = beachline.right_edge(arc_idx).unwrap();

    segments.push([xl.origin, s]);
    segments.push([xr.origin, s]);

    let x = BreakPoint::new(s, normal_vector(l.site - r.site), l.site, r.site);
    beachline.replace_breakpoint(xl_idx, arc_idx, xr_idx, x, eq, yl);
}

fn remove_circle_event(arc_idx: usize, eq: &mut EventQueue) {
    let circle = Event::Circle(arc_idx);
    eq.remove(&circle);
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{compare_edges, compare_segments};

    use super::*;

    #[test]
    fn vertical_line() {
        let bounding_box = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let sites = vec![
            Point::new(250.0.into(), 500.0.into()),
            Point::new(750.0.into(), 500.0.into()),
        ];

        let voronoi = fortunes_algorithm(&sites, &bounding_box);

        assert_eq!(voronoi.len(), 2);

        let gold = [
            [
                Point::new(500.0.into(), 500.0.into()),
                Point::new(500.0.into(), 1000.0.into()),
            ],
            [
                Point::new(500.0.into(), 500.0.into()),
                Point::new(500.0.into(), 0.0.into()),
            ],
        ];

        assert!(compare_edges(&gold, &voronoi));
    }

    #[test]
    fn horizontal_line() {
        let bounding_box = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let sites = vec![
            Point::new(500.0.into(), 250.0.into()),
            Point::new(500.0.into(), 750.0.into()),
        ];

        let voronoi = fortunes_algorithm(&sites, &bounding_box);

        assert_eq!(voronoi.len(), 2);

        let gold = [
            [
                Point::new(500.0.into(), 500.0.into()),
                Point::new(1000.0.into(), 500.0.into()),
            ],
            [
                Point::new(500.0.into(), 500.0.into()),
                Point::new(0.0.into(), 500.0.into()),
            ],
        ];

        assert!(compare_edges(&gold, &voronoi));
    }

    #[test]
    fn three_points() {
        let bounding_box = BoundingBox::new(0.0.into(), 1000.0.into(), 0.0.into(), 1000.0.into());

        let sites = vec![
            Point::new(250.0.into(), 250.0.into()),
            Point::new(500.0.into(), 750.0.into()),
            Point::new(750.0.into(), 250.0.into()),
        ];

        let voronoi = fortunes_algorithm(&sites, &bounding_box);

        assert_eq!(voronoi.len(), 3);

        let gold = [
            [
                Point::new(500.0.into(), 437.5.into()),
                Point::new(500.0.into(), 0.0.into()),
            ],
            [
                Point::new(500.0.into(), 437.5.into()),
                Point::new(1000.0.into(), 687.5.into()),
            ],
            [
                Point::new(500.0.into(), 437.5.into()),
                Point::new(0.0.into(), 687.5.into()),
            ],
        ];

        assert!(compare_edges(&gold, &voronoi));
    }
}
