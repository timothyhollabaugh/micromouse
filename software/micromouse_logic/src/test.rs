//! Various functions to help writing tests

use crate::math::Vector;

const MAX_DELTA: f32 = 0.00002;

/// Assert that the two floats are close enough to be equal
pub fn assert_close(left: f32, right: f32) {
    if left.is_infinite()
        && left.is_sign_positive()
        && right.is_infinite()
        && right.is_sign_positive()
    {
        return;
    }

    if left.is_infinite()
        && left.is_sign_negative()
        && right.is_infinite()
        && right.is_sign_negative()
    {
        return;
    }

    let delta = (left - right).abs();
    assert!(
        delta <= MAX_DELTA,
        "\nleft: {}\nright: {}\ndelta: {}\n",
        left,
        right,
        delta
    );
}

/// Assert that the two float vectors are close enough to be equal
pub fn assert_close2(left: Vector, right: Vector) {
    let delta0 = (left.x - right.x).abs();
    let delta1 = (left.y - right.y).abs();
    assert!(
        delta0 <= MAX_DELTA && delta1 <= MAX_DELTA,
        "\nleft: {:?}\nright: {:?}\ndelta: {:?}\n",
        left,
        right,
        (delta0, delta1),
    );
}
