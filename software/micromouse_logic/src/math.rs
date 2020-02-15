//! Various math related types

use core::f32::consts::PI;

use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use core::ops::Neg;

/// A 2d vector
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn magnitude(&self) -> f32 {
        F32Ext::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn direction(&self) -> Direction {
        Direction::from(F32Ext::atan2(self.y, self.x))
    }

    pub fn cross(&self, v: Vector) -> f32 {
        self.x * v.y - v.x * self.y
    }

    pub fn dot(&self, v: Vector) -> f32 {
        self.x * v.x + self.y * v.y
    }

    /// Project `self` onto vector `v`
    pub fn project_onto(&self, v: Vector) -> Vector {
        //(self.dot(v) / v.dot(v)) * v
        Vector {
            x: (self.x * v.x * v.x + self.y * v.y * v.x) / (v.x * v.x + v.y * v.y),
            y: (self.x * v.x * v.y + self.y * v.y * v.y) / (v.x * v.x + v.y * v.y),
        }
    }

    /// Rotate about the origin `theta` radians
    pub fn rotated(&self, theta: Direction) -> Vector {
        Vector {
            x: self.x * F32Ext::cos(f32::from(theta))
                - self.y * F32Ext::sin(f32::from(theta)),
            y: self.x * F32Ext::sin(f32::from(theta))
                + self.y * F32Ext::cos(f32::from(theta)),
        }
    }
}

#[cfg(test)]
mod vector_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use core::f32::consts::FRAC_PI_4;
    use core::f32::consts::SQRT_2;

    use super::Vector;
    use crate::math::{DIRECTION_0, DIRECTION_PI_2};

    #[test]
    fn vector_magnitude_test() {
        assert_close(Vector { x: 1.0, y: 1.0 }.magnitude(), SQRT_2);
    }

    #[test]
    fn vector_direction_test() {
        assert_close(f32::from(Vector { x: 1.0, y: 1.0 }.direction()), FRAC_PI_4);
    }

    #[test]
    fn vector_dot_test_parallel() {
        assert_close(
            Vector { x: 1.0, y: 1.0 }.dot(Vector { x: 1.0, y: 1.0 }),
            2.0,
        );
    }

    #[test]
    fn vector_dot_test_perpendicular() {
        assert_close(
            Vector { x: 1.0, y: 1.0 }.dot(Vector { x: -1.0, y: 1.0 }),
            0.0,
        );
    }

    #[test]
    fn vector_cross_test_parallel() {
        assert_close(
            Vector { x: 1.0, y: 1.0 }.cross(Vector { x: 1.0, y: 1.0 }),
            0.0,
        )
    }

    #[test]
    fn vector_cross_test_perpendicular() {
        assert_close(
            Vector { x: 1.0, y: 1.0 }.cross(Vector { x: -1.0, y: 1.0 }),
            2.0,
        )
    }

    #[test]
    fn vector_project_onto_test() {
        assert_close2(
            Vector { x: 2.0, y: 0.0 }.project_onto(Vector { x: 2.0, y: 2.0 }),
            Vector { x: 1.0, y: 1.0 },
        )
    }

    #[test]
    fn vector_rotated() {
        assert_close2(
            Vector { x: 1.0, y: 0.0 }.rotated(DIRECTION_PI_2),
            Vector { x: 0.0, y: 1.0 },
        )
    }

    #[test]
    fn vector_rotated2() {
        assert_close2(
            Vector { x: 30.0, y: 32.0 }.rotated(DIRECTION_0),
            Vector { x: 30.0, y: 32.0 },
        )
    }
}

impl core::ops::Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl core::ops::Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl core::ops::Mul<Vector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl core::ops::Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl core::ops::AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

/// A direction wrapped to 0 - 2pi
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Direction(f32);

impl Direction {
    pub fn centered_at(self, center: Direction) -> f32 {
        let raw_self = f32::from(self);
        let raw_center = f32::from(center);

        let diff = raw_self - raw_center;

        if diff > PI {
            raw_self - 2.0 * PI
        } else if diff < -PI {
            raw_self + 2.0 * PI
        } else {
            raw_self
        }
    }

    pub fn into_unit_vector(self) -> Vector {
        Vector {
            x: F32Ext::cos(self.0),
            y: F32Ext::sin(self.0),
        }
    }
}

impl From<f32> for Direction {
    fn from(other: f32) -> Direction {
        Direction((other + 2.0 * PI) % (2.0 * PI))
    }
}

impl From<Direction> for f32 {
    fn from(other: Direction) -> f32 {
        other.0
    }
}

impl From<Direction> for f64 {
    fn from(other: Direction) -> f64 {
        other.0 as f64
    }
}

impl core::ops::AddAssign for Direction {
    fn add_assign(&mut self, rhs: Self) {
        *self = Direction::from(self.0 + rhs.0)
    }
}

impl core::ops::Add for Direction {
    type Output = Direction;

    fn add(self, rhs: Self) -> Self::Output {
        Direction::from(self.0 + rhs.0)
    }
}

impl core::ops::Add<f32> for Direction {
    type Output = Direction;
    fn add(self, rhs: f32) -> Self::Output {
        Direction::from(self.0 + rhs)
    }
}

impl core::ops::Add<Direction> for f32 {
    type Output = Direction;
    fn add(self, rhs: Direction) -> Self::Output {
        Direction::from(self + rhs.0)
    }
}

impl core::ops::Sub for Direction {
    type Output = Direction;

    fn sub(self, rhs: Self) -> Self::Output {
        Direction::from(self.0 - rhs.0)
    }
}

impl core::ops::Div<f32> for Direction {
    type Output = Direction;

    fn div(self, rhs: f32) -> Self::Output {
        Direction::from(self.0 / rhs)
    }
}

impl core::ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        Direction::from(2.0 * PI - self.0)
    }
}

pub const DIRECTION_0: Direction = Direction(0.0);
pub const DIRECTION_PI_2: Direction = Direction(core::f32::consts::FRAC_PI_2);
pub const DIRECTION_PI: Direction = Direction(core::f32::consts::PI);
pub const DIRECTION_3_PI_2: Direction = Direction(3.0 * core::f32::consts::FRAC_PI_2);

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Orientation {
    pub position: Vector,
    pub direction: Direction,
}

impl Orientation {
    pub fn update_from_encoders(
        self,
        config: &MechanicalConfig,
        delta_left: i32,
        delta_right: i32,
    ) -> Orientation {
        let delta_linear = config.ticks_to_mm((delta_right + delta_left) as f32 / 2.0);
        let delta_angular = config.ticks_to_rads((delta_right - delta_left) as f32 / 2.0);

        let mid_dir = f32::from(self.direction) + delta_angular / 2.0;

        Orientation {
            position: Vector {
                x: self.position.x + delta_linear * F32Ext::cos(mid_dir),
                y: self.position.y + delta_linear * F32Ext::sin(mid_dir),
            },
            direction: self.direction + Direction::from(delta_angular),
        }
    }

    pub fn offset(self, offset: Orientation) -> Orientation {
        Orientation {
            position: self.position + offset.position.rotated(self.direction),
            direction: self.direction + offset.direction,
        }
    }
}

#[cfg(test)]
mod orientation_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::math::{Orientation, Vector, DIRECTION_0, DIRECTION_PI_2};

    #[test]
    fn offset() {
        let orientation = Orientation {
            position: Vector { x: 1.0, y: 0.0 },
            direction: DIRECTION_PI_2,
        };

        let offset_orientation = Orientation {
            position: Vector { x: 0.5, y: 0.5 },
            direction: DIRECTION_PI_2 / 2.0,
        };

        let result_orientation = orientation.offset(offset_orientation);

        assert_close2(result_orientation.position, Vector { x: 0.5, y: 0.5 });
        assert_close(
            f32::from(result_orientation.direction),
            f32::from(DIRECTION_PI_2 + DIRECTION_PI_2 / 2.0),
        )
    }

    #[test]
    fn offset2() {
        let orientation = Orientation {
            position: Vector {
                x: 1260.0,
                y: 1170.0,
            },
            direction: DIRECTION_0,
        };

        let offset_orientation = Orientation {
            position: Vector { x: 30.0, y: 32.0 },
            direction: DIRECTION_PI_2,
        };

        let result_orientation = orientation.offset(offset_orientation);

        assert_close2(
            result_orientation.position,
            Vector {
                x: 1260.0 + 30.0,
                y: 1170.0 + 32.0,
            },
        );
        assert_close(
            f32::from(result_orientation.direction),
            f32::from(DIRECTION_PI_2),
        )
    }
}
