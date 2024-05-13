use core::panic;

use ordered_float::OrderedFloat;

use crate::{geometry::*, Event, EventQueue};

pub struct Beachline {
    root: Option<usize>,
    nodes: Vec<BeachlineEntry>,
}

pub struct BeachlineEntry {
    left_child: Option<usize>,
    right_child: Option<usize>,
    parent: Option<usize>,
    data: BeachlineData,
}

impl BeachlineEntry {
    fn new(
        left_child: Option<usize>,
        right_child: Option<usize>,
        parent: Option<usize>,
        data: BeachlineData,
    ) -> Self {
        Self {
            left_child,
            right_child,
            parent,
            data,
        }
    }
}

pub enum BeachlineData {
    BreakPoint(BreakPoint),
    Arc(Arc),
}

pub struct BreakPoint {
    pub origin: Point,
    direction: Point,
    l: Point,
    r: Point,
}

impl BreakPoint {
    pub fn new(origin: Point, direction: Point, l: Point, r: Point) -> Self {
        Self {
            origin,
            direction,
            l,
            r,
        }
    }
}

pub struct Arc {
    pub site: Point,
}

impl Arc {
    pub fn new(site: Point) -> Self {
        Self { site }
    }
}

impl Beachline {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: vec![],
        }
    }

    pub fn add_first_parabola(self: &mut Self, p: Point) {
        assert!(self.root.is_none());

        let entry = BeachlineEntry::new(None, None, None, BeachlineData::Arc(Arc::new(p)));
        self.root = Some(self.nodes.len());
        self.nodes.push(entry);
    }

    pub fn arc_under_point(self: &Self, p: &Point, yl: OrderedFloat<f64>) -> Option<(&Arc, usize)> {
        self.root.map(|mut curr_idx| loop {
            let node = &self.nodes[curr_idx];
            match &node.data {
                BeachlineData::BreakPoint(bp) => {
                    let x = breakpoint_at_x(&bp.l, &bp.r, yl);
                    if x < p.x {
                        curr_idx = node.left_child.unwrap();
                    } else {
                        curr_idx = node.right_child.unwrap();
                    }
                }
                BeachlineData::Arc(arc) => return (arc, curr_idx),
            }
        })
    }

    pub fn replace_arc(
        self: &mut Self,
        arc_idx: usize,
        a: Arc,
        xl: BreakPoint,
        b: Arc,
        xr: BreakPoint,
        c: Arc,
        eq: &mut EventQueue,
        yl: OrderedFloat<f64>,
    ) {
        let parent = self.nodes[arc_idx].parent;

        let a_idx = self.nodes.len();
        let xl_idx = a_idx + 1;
        let b_idx = a_idx + 2;
        let xr_idx = a_idx + 3;
        let c_idx = a_idx + 4;

        let a_entry = BeachlineEntry::new(None, None, Some(xl_idx), BeachlineData::Arc(a));
        let xl_entry = BeachlineEntry::new(
            Some(a_idx),
            Some(xr_idx),
            parent,
            BeachlineData::BreakPoint(xl),
        );
        let b_entry = BeachlineEntry::new(None, None, Some(xr_idx), BeachlineData::Arc(b));
        let xr_entry = BeachlineEntry::new(
            Some(b_idx),
            Some(c_idx),
            Some(xl_idx),
            BeachlineData::BreakPoint(xr),
        );
        let c_entry = BeachlineEntry::new(None, None, Some(xr_idx), BeachlineData::Arc(c));

        self.nodes.push(a_entry);
        self.nodes.push(xl_entry);
        self.nodes.push(b_entry);
        self.nodes.push(xr_entry);
        self.nodes.push(c_entry);

        if let Some(parent_idx) = parent {
            let parent_node = &mut self.nodes[parent_idx];
            if parent_node.left_child.unwrap() == arc_idx {
                parent_node.left_child = Some(xl_idx);
            } else {
                parent_node.right_child = Some(xl_idx);
            }
        } else {
            self.root = Some(xl_idx);
        }

        self.check_circle_event(a_idx, eq, yl);
        self.check_circle_event(c_idx, eq, yl);
    }

    pub fn replace_breakpoint(
        self: &mut Self,
        xl_idx: usize,
        p_idx: usize,
        xr_idx: usize,
        x: BreakPoint,
        eq: &mut EventQueue,
        yl: OrderedFloat<f64>,
    ) {
        let x_idx = self.nodes.len();

        let (_, l_idx) = self
            .left_arc(p_idx)
            .expect("replace_breakpoint: left arc not found");
        let (_, r_idx) = self
            .right_arc(p_idx)
            .expect("replace_breakpoint: right arc not found");

        let parent_idx = self.nodes[p_idx]
            .parent
            .expect("repalce_breakpoint: parent not found");
        let parent_node = &self.nodes[parent_idx];
        let (sister_idx, cousin_idx) = if parent_node.left_child.unwrap() == p_idx {
            (r_idx, l_idx)
        } else {
            (l_idx, r_idx)
        };

        let pred_idx = self
            .predecessor(p_idx)
            .expect("repalce_breakpoint: predecessor not found");
        let succ_idx = self
            .successor(p_idx)
            .expect("replace_breakpoint: successor not found");

        let pred_node = &self.nodes[pred_idx];
        let succ_node = &self.nodes[succ_idx];

        let (granny_idx, granny_node) = if parent_idx == pred_idx {
            (succ_idx, succ_node)
        } else {
            (pred_idx, pred_node)
        };

        let x_entry = BeachlineEntry::new(
            Some(l_idx),
            Some(r_idx),
            granny_node.parent,
            BeachlineData::BreakPoint(x),
        );

        if let Some(root_idx) = granny_node.parent {
            let root_node = &mut self.nodes[root_idx];
            if root_node.left_child.unwrap() == granny_idx {
                root_node.left_child = Some(x_idx);
            } else {
                root_node.right_child = Some(x_idx);
            }
        }

        self.nodes.push(x_entry);

        self.check_circle_event(l_idx, eq, yl);
        self.check_circle_event(r_idx, eq, yl);
    }

    fn check_circle_event(self: &Self, arc_idx: usize, eq: &mut EventQueue, yl: OrderedFloat<f64>) {
        let p = self.arc(arc_idx);
        let l_opt = self.left_arc(arc_idx);
        let r_opt = self.right_arc(arc_idx);
        let xl_opt = self.left_edge(arc_idx);
        let xr_opt = self.right_edge(arc_idx);

        match (l_opt, r_opt) {
            (Some((l, _)), Some((r, _))) => {
                if l.site == r.site {
                    return;
                }
                let (xl, _) = xl_opt.unwrap();
                let (xr, _) = xr_opt.unwrap();
                if let Some(s) = intersection(&xl.origin, &xl.direction, &xr.origin, &xr.direction)
                {
                    let r = distance(&p.site, &s);
                    let circle_top = s.y - r;
                    if circle_top > yl {
                        return;
                    }
                    eq.push(Event::Circle(arc_idx), circle_top);
                } else {
                    return;
                }
            }
            _ => return,
        }
    }

    fn minimum(self: &Self, mut curr_idx: usize) -> usize {
        loop {
            let node = &self.nodes[curr_idx];
            match &node.data {
                BeachlineData::BreakPoint(..) => {
                    curr_idx = node.left_child.unwrap();
                }
                BeachlineData::Arc(..) => return curr_idx,
            }
        }
    }

    fn maximum(self: &Self, mut curr_idx: usize) -> usize {
        loop {
            let node = &self.nodes[curr_idx];
            match &node.data {
                BeachlineData::BreakPoint(..) => {
                    curr_idx = node.right_child.unwrap();
                }
                BeachlineData::Arc(..) => return curr_idx,
            }
        }
    }

    fn predecessor(self: &Self, mut curr_idx: usize) -> Option<usize> {
        let mut curr_node = &self.nodes[curr_idx];
        loop {
            if let Some(parent_idx) = curr_node.parent {
                let parent_node = &self.nodes[parent_idx];
                let left_idx = parent_node.left_child.unwrap();
                if left_idx == curr_idx {
                    curr_idx = parent_idx;
                    curr_node = parent_node;
                } else {
                    return Some(curr_idx);
                }
            } else {
                return None;
            }
        }
    }

    fn successor(self: &Self, mut curr_idx: usize) -> Option<usize> {
        let mut curr_node = &self.nodes[curr_idx];
        loop {
            if let Some(parent_idx) = curr_node.parent {
                let parent_node = &self.nodes[parent_idx];
                let right_idx = parent_node.right_child.unwrap();
                if right_idx == curr_idx {
                    curr_idx = parent_idx;
                    curr_node = parent_node;
                } else {
                    return Some(curr_idx);
                }
            } else {
                return None;
            }
        }
    }

    pub fn left_arc(self: &Self, arc_idx: usize) -> Option<(&Arc, usize)> {
        self.predecessor(arc_idx)
            .and_then(|succ| self.nodes[succ].left_child)
            .map(|left_idx| {
                let arc_idx = self.maximum(left_idx);
                (self.arc(arc_idx), arc_idx)
            })
    }

    pub fn right_arc(self: &Self, arc_idx: usize) -> Option<(&Arc, usize)> {
        self.successor(arc_idx)
            .and_then(|succ| self.nodes[succ].right_child)
            .map(|right_idx| {
                let arc_idx = self.minimum(right_idx);
                (self.arc(arc_idx), arc_idx)
            })
    }

    pub fn arc(self: &Self, idx: usize) -> &Arc {
        match &self.nodes[idx].data {
            BeachlineData::BreakPoint(..) => panic!("not an arc"),
            BeachlineData::Arc(arc) => arc,
        }
    }

    pub fn breakpoint(self: &Self, idx: usize) -> &BreakPoint {
        match &self.nodes[idx].data {
            BeachlineData::BreakPoint(bp) => bp,
            BeachlineData::Arc(..) => panic!("not a breakpoint"),
        }
    }

    pub fn left_edge(self: &Self, arc_idx: usize) -> Option<(&BreakPoint, usize)> {
        self.predecessor(arc_idx)
            .and_then(|pred| match &self.nodes[pred].data {
                BeachlineData::BreakPoint(bp) => Some((bp, pred)),
                _ => None,
            })
    }

    pub fn right_edge(self: &Self, arc_idx: usize) -> Option<(&BreakPoint, usize)> {
        self.successor(arc_idx)
            .and_then(|succ| match &self.nodes[succ].data {
                BeachlineData::BreakPoint(bp) => Some((bp, succ)),
                _ => None,
            })
    }

    pub fn extend_edges_to_bounding_box(
        self: &Self,
        bounding_box: &BoundingBox,
        edges: &mut Vec<Segment>,
    ) {
        self.extend_edges_to_bounding_box_aux(bounding_box, edges, self.root);
    }

    fn extend_edges_to_bounding_box_aux(
        self: &Self,
        bounding_box: &BoundingBox,
        edges: &mut Vec<Segment>,
        root: Option<usize>,
    ) {
        if let Some(root_idx) = root {
            let node = &self.nodes[root_idx];
            match &node.data {
                BeachlineData::BreakPoint(bp) => {
                    edges.push(bounded_segment(&bp.origin, &bp.direction, bounding_box))
                }
                _ => {}
            }
            self.extend_edges_to_bounding_box_aux(bounding_box, edges, node.left_child);
            self.extend_edges_to_bounding_box_aux(bounding_box, edges, node.right_child);
        }
    }
}
