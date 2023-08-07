use std::f64::consts::PI;

use crate::base::Rect;

const EPSILON: f64 = 0.0001;

/// Compare two values
/// # Arguments
///
///  epsilon: it is a small threshold used to determine their approximate equality,
///  comparing the absolute difference between the two numbers with this threshold.
///
/// # Returns
///
///  0: same
///
///  1: first is bigger
///
/// -1: second is bigger
pub fn compare(first: f64, second: f64, epsilon: f64) -> i8 {
    let takeaway = first - second;
    if f64::abs(takeaway) < epsilon {
        return 0;
    }

    if takeaway > 0.0 && takeaway >= epsilon {
        return 1;
    }

    if takeaway < 0.0 && takeaway <= epsilon {
        return -1;
    }

    unreachable!()
}

pub fn is_between(value: f64, bound_1: f64, bound_2: f64, tolerance: f64) -> bool {
    if compare(bound_2, bound_1, EPSILON) == 1 {
        if value >= bound_1 - tolerance && value <= bound_2 + tolerance {
            return true;
        }
    } else if value >= bound_2 - tolerance && value <= bound_1 + tolerance {
        return true;
    }

    false
}

pub fn abs_angle(center_x: f64, center_y: f64, another_x: f64, another_y: f64) -> f64 {
    let distance_x = f64::abs(another_x - center_x);
    let distance_y = f64::abs(another_y - center_y);

    let mut angle = f64::atan2(distance_y, distance_x) * 180.0 / PI;

    if (compare(another_x, center_x, EPSILON) == 1 || compare(another_x, center_x, EPSILON) == 0)
        && (compare(another_y, center_y, EPSILON) == 1
            || compare(another_y, center_y, EPSILON) == 0)
    {
        return angle;
    }

    if compare(center_x, another_x, EPSILON) == 1
        && (compare(another_y, center_y, EPSILON) == 1
            || compare(another_y, center_y, EPSILON) == 0)
    {
        angle = 180.0 - angle;
        return angle;
    }

    if (compare(center_x, another_x, EPSILON) == 1 || compare(center_x, another_x, EPSILON) == 0)
        && compare(center_y, another_y, EPSILON) == 1
    {
        angle += 180.0;
        return angle;
    }

    if compare(another_x, center_x, EPSILON) == 1 && compare(center_y, another_y, EPSILON) == 1 {
        angle = 360.0 - angle;
        return angle;
    }

    unreachable!()
}

pub fn rotate(
    angle: f64,
    center_x: f64,
    center_y: f64,
    another_x: f64,
    another_y: f64,
) -> (f64, f64) {
    let cos = f64::cos(angle * PI / 180.0);
    let sin = f64::sin(angle * PI / 180.0);

    let temp_x = another_x - center_x;
    let temp_y = another_y - center_y;

    let temp_x2 = cos * temp_x - sin * temp_y;
    let temp_y2 = sin * temp_x + cos * temp_y;

    (temp_x2 + center_x, temp_y2 + center_y)
}

pub fn check_point_lies_on_line(
    point: (f64, f64),
    start: (f64, f64),
    end: (f64, f64),
    tolerance: f64,
) -> bool {
    if !is_between(point.0, start.0, end.0, tolerance)
        || !is_between(point.1, start.1, end.1, tolerance)
    {
        return false;
    }

    if compare(f64::abs(end.0 - start.0), 0.0, EPSILON) == 0 {
        //Vertical line.
        return true;
    }

    let angle = abs_angle(start.0, start.1, end.0, end.1);
    let point_angle = abs_angle(start.0, start.1, point.0, point.1);

    let (rotated_x, rotated_y) = rotate(360.0 - point_angle, start.0, start.1, point.0, point.1);

    let (rotated_x, rotated_y) = rotate(angle, start.0, start.1, rotated_x, rotated_y);

    if rotated_x - tolerance <= point.0
        && rotated_x + tolerance >= point.0
        && rotated_y - tolerance <= point.1
        && rotated_y + tolerance >= point.1
    {
        return true;
    }

    false
}

pub fn check_point_lies_inside_rect(point: (f64, f64), rect: Rect, tolerance: f64) -> bool {
    let bottom_right = (rect.top_left.0 + rect.width, rect.top_left.1 - rect.height);

    point.0 >= rect.top_left.0 - tolerance
        && point.0 <= bottom_right.0 + tolerance
        && point.1 <= rect.top_left.1 + tolerance
        && point.1 >= bottom_right.1 - tolerance
}

pub fn check_two_line_segments_intersect(
    start_1: (f64, f64),
    end_1: (f64, f64),
    start_2: (f64, f64),
    end_2: (f64, f64),
) -> Option<(f64, f64)> {
    let s1_x = end_1.0 - start_1.0;
    let s1_y = end_1.1 - start_1.1;
    let s2_x = end_2.0 - start_2.0;
    let s2_y = end_2.1 - start_2.1;

    let s = (-s1_y * (start_1.0 - start_2.0) + s1_x * (start_1.1 - start_2.1))
        / (-s2_x * s1_y + s1_x * s2_y);
    let t = (s2_x * (start_1.1 - start_2.1) - s2_y * (start_1.0 - start_2.0))
        / (-s2_x * s1_y + s1_x * s2_y);

    if (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t) {
        return Some((start_1.0 + (t * s1_x), start_1.1 + (t * s1_y)));
    }

    None
}

/// Caculate rectangle point with two points.
/// # Arguments
///
/// # Returns
///
///  rect
pub fn caculate_rectangle(
    first: (f64, f64),
    second: (f64, f64),
    y_axis_increase_downward: bool,
) -> Rect {
    let (left_x, right_x) = if compare(first.0, second.0, EPSILON) == 1 {
        (second.0, first.0)
    } else {
        (first.0, second.0)
    };

    let (top_y, bottom_y) = if compare(first.1, second.1, EPSILON) == 1 {
        if y_axis_increase_downward {
            (second.1, first.1)
        } else {
            (first.1, second.1)
        }
    } else if y_axis_increase_downward {
        (first.1, second.1)
    } else {
        (second.1, first.1)
    };

    if y_axis_increase_downward {
        Rect::new((left_x, top_y), right_x - left_x, bottom_y - top_y)
    } else {
        Rect::new((left_x, top_y), right_x - left_x, top_y - bottom_y)
    }
}
