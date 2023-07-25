use std::f64::consts::PI;

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
