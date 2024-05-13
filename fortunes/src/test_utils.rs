use approx::relative_eq;
use itertools::Itertools;

use crate::{Point, Segment};

pub fn compare_points(a: &Point, b: &Point) -> bool {
    relative_eq!(a.x.into_inner(), b.x.into_inner())
        && relative_eq!(a.y.into_inner(), b.y.into_inner())
}

pub fn compare_segments(a: &Segment, b: &Segment) -> bool {
    (compare_points(&a[0], &b[0]) && compare_points(&a[1], &b[1]))
        || (compare_points(&a[0], &b[1]) && compare_points(&a[1], &b[0]))
}

pub fn compare_edges(a: &[Segment], b: &[Segment]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for perm in a.iter().permutations(a.len()).unique() {
        let mut found_incorrect = false;
        for i in 0..perm.len() {
            if !compare_segments(perm[i], &b[i]) {
                found_incorrect = true;
                break;
            }
        }
        if !found_incorrect {
            return true;
        }
    }

    false
}
