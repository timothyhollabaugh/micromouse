//! Bezier curves

use core::cmp::Ordering;

use serde::Deserialize;
use serde::Serialize;

use crate::math::{Direction, Vector};

/// Make sure `n` is between `min` and `max`
pub fn clamp(n: f32, min: f32, max: f32) -> f32 {
    if n > max {
        max
    } else if n < min {
        min
    } else {
        n
    }
}

pub trait Curve {
    type Derivative: Curve;

    /// Evaluate the curve at `t`
    fn at(&self, t: f32) -> Vector;

    /// The derivative
    fn derivative(&self) -> Self::Derivative;

    /// The curvature
    fn curvature(&self, t: f32) -> f32 {
        let d1 = self.derivative().at(t);
        let d2 = self.derivative().derivative().at(t);

        let d1_magnitude = d1.magnitude();

        (d1.x * d2.y - d2.x * d1.y)
            / (d1_magnitude * d1_magnitude * d1_magnitude)
    }

    /// The closest point on the curve
    ///
    /// If `m` is past either end of the curve, the curve gets extended with a line tangent to the
    /// curve at that end and the closest point on that line is found. The returned `t` be greater
    /// than 1.0 if it is past the end, or less than 0.0 if it is before the start
    ///
    /// By default, it does a binary search with default parameters,
    /// but can be overridden if there is a better method
    fn closest_point(&self, m: Vector) -> (f32, Vector) {
        // Check if the point is before the start
        let start_point = self.at(0.0);
        let start_tangent = self.derivative().at(0.0);
        let start_normal = Vector {
            x: -start_tangent.y,
            y: start_tangent.x,
        };
        if start_normal.cross(m - start_point) > 0.0 {
            let line = Line {
                start: start_point - start_tangent,
                end: start_point,
            };
            let (_, p) = line.closest_point(m);
            return (-0.1, p);
        }

        // Check if the point is after the end
        let end_point = self.at(1.0);
        let end_tangent = self.derivative().at(1.0);
        let end_normal = Vector {
            x: -end_tangent.y,
            y: end_tangent.x,
        };
        if end_normal.cross(m - end_point) < 0.0 {
            let line = Line {
                start: end_point + end_tangent,
                end: end_point,
            };
            let (_, p) = line.closest_point(m);
            return (1.1, p);
        }

        self.closest_point_by_binary_search(m, 32, 0.000001)
    }

    /// Do a binary search to find the closest point on the curve.
    /// Useful for curves like beziers where there is no other good way.
    ///
    /// The search is done in two phases: A coarse linear search and a fine binary search.
    /// The coarse search finds a close value to start the binary search at.
    ///
    /// The `steps` is how may pieces to divide the curve into for the coarse search. Higher values
    /// will result in a longer linear search, but a shorter binary search. The `epsilon` is how
    /// close the binary search needs to get before it is done. Higher values will be quicker, but
    /// less accurate. If `steps` is 0, the binary search will start at t=0.5
    ///
    /// Returns a tuple of `(t, point)` for the closest point
    fn closest_point_by_binary_search(
        &self,
        m: Vector,
        steps: u16,
        epsilon: f32,
    ) -> (f32, Vector) {
        // Do a coarse linear search to get a good starting point for the binary search
        let mut current = (0..steps)
            // Compute the point and distance at each t
            .map(|i| {
                let t = i as f32 / steps as f32;
                let p = self.at(t);
                let d = (m - p).magnitude();
                (t, p, d)
            })
            // Find the closest point
            .min_by(|&(_, _, d1), &(_, _, d2)| {
                if d1 < d2 {
                    Ordering::Less
                } else if d1 > d2 {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            // If steps was 0 and no point was found, start in the middle
            .unwrap_or((0.5, self.at(0.5), (m - self.at(0.5)).magnitude()));

        let mut h = 1.0 / steps as f32;

        loop {
            let (t, p, d) = current;

            if h < epsilon {
                break (t, p);
            }

            let t1 = t + h;
            let p1 = self.at(t1);
            let d1 = (p1 - m).magnitude();

            let t2 = t - h;
            let p2 = self.at(t2);
            let d2 = (p2 - m).magnitude();

            if d1 < d && d1 < d2 {
                current = (t1, p1, d1);
            } else if d2 < d && d2 < d1 {
                current = (t2, p2, d2);
            } else {
                h /= 2.0;
            }
        }
    }
}

/// A circular arc
pub struct Arc {
    pub center: Vector,
    pub start_dir: Direction,
    pub theta: f32,
    pub radius: f32,
}

impl Curve for Arc {
    type Derivative = Arc;

    fn at(&self, t: f32) -> Vector {
        let theta = Direction::from(self.theta * t) + self.start_dir;
        self.radius * theta.into_unit_vector() + self.center
    }

    fn derivative(&self) -> Arc {
        Arc {
            center: Vector { x: 0.0, y: 0.0 },
            start_dir: self.start_dir,
            theta: self.theta,
            radius: self.radius * self.theta,
        }
    }

    fn curvature(&self, _t: f32) -> f32 {
        return 1.0 / self.radius;
    }

    fn closest_point(&self, m: Vector) -> (f32, Vector) {
        let m_dir = (m - self.center).direction();

        let m_theta = f32::from(m_dir - self.start_dir);

        let t = m_theta / self.theta;

        let t = clamp(t, 0.0, 1.0);

        (t, self.at(t))
    }
}

#[cfg(test)]
mod arc_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::Arc;
    use crate::bezier::Curve;
    use crate::math::{Vector, DIRECTION_3_PI_2};
    use core::f32::consts::{FRAC_PI_2, PI, SQRT_2};

    const A: Arc = Arc {
        center: Vector { x: 0.0, y: 2.0 },
        start_dir: DIRECTION_3_PI_2,
        theta: FRAC_PI_2,
        radius: 2.0,
    };

    #[test]
    fn start() {
        assert_close2(A.at(0.0), Vector { x: 0.0, y: 0.0 });
    }

    #[test]
    fn end() {
        assert_close2(A.at(1.0), Vector { x: 2.0, y: 2.0 });
    }

    #[test]
    fn mid() {
        assert_close2(
            A.at(0.5),
            Vector {
                x: SQRT_2,
                y: 2.0 - SQRT_2,
            },
        )
    }

    #[test]
    fn derivative() {
        let d = A.derivative();
        assert_close2(d.center, Vector { x: 0.0, y: 0.0 });
        assert_close(d.radius, PI);
        assert_close(d.theta, FRAC_PI_2);
        assert_close(f32::from(d.start_dir), f32::from(DIRECTION_3_PI_2))
    }

    #[test]
    fn curvature() {
        assert_close(A.curvature(0.5), 0.5);
    }

    #[test]
    fn closest_point() {
        let (t, p) = A.closest_point(Vector { x: 1.75, y: 0.25 });
        assert_close(t, 0.5);
        assert_close2(
            p,
            Vector {
                x: SQRT_2,
                y: 2.0 - SQRT_2,
            },
        )
    }
}

impl Curve for Vector {
    type Derivative = Vector;

    fn at(&self, _t: f32) -> Vector {
        *self
    }

    fn derivative(&self) -> Vector {
        Vector { x: 0.0, y: 0.0 }
    }

    fn curvature(&self, _t: f32) -> f32 {
        return 0.0;
    }

    fn closest_point(&self, _m: Vector) -> (f32, Vector) {
        (0.0, *self)
    }
}

pub struct Line {
    pub start: Vector,
    pub end: Vector,
}

impl Curve for Line {
    type Derivative = Vector;

    fn at(&self, t: f32) -> Vector {
        Vector {
            x: self.start.x * (1.0 - t) + self.end.x * t,
            y: self.start.y * (1.0 - t) + self.end.y * t,
        }
    }

    fn derivative(&self) -> Self::Derivative {
        self.end - self.start
    }

    fn curvature(&self, _t: f32) -> f32 {
        return 0.0;
    }

    fn closest_point(&self, m: Vector) -> (f32, Vector) {
        let p = (m - self.start).project_onto(self.derivative());
        let t = p.x / self.derivative().x;
        (t, p + self.start)
    }
}

#[cfg(test)]
mod line_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::bezier::Curve;
    use crate::bezier::Line;
    use crate::math::Vector;

    const B: Line = Line {
        start: Vector { x: 0.0, y: 0.0 },
        end: Vector { x: 1.0, y: 1.0 },
    };

    #[test]
    fn start_is_t0() {
        assert_close2(B.at(0.0), Vector { x: 0.0, y: 0.0 });
    }

    #[test]
    fn end_is_t1() {
        assert_close2(B.at(1.0), Vector { x: 1.0, y: 1.0 });
    }

    #[test]
    fn mid() {
        assert_close2(B.at(0.5), Vector { x: 0.5, y: 0.5 });
    }

    #[test]
    fn closest_point() {
        let (t, p) = B.closest_point(Vector { x: 0.75, y: 0.25 });
        assert_close(t, 0.5);
        assert_close2(p, Vector { x: 0.5, y: 0.5 });
    }
}

pub struct Bezier2 {
    pub start: Vector,
    pub ctrl0: Vector,
    pub end: Vector,
}

impl Curve for Bezier2 {
    type Derivative = Line;

    /// Evaluate the curve at `t`
    fn at(&self, t: f32) -> Vector {
        Vector {
            x: self.start.x * (1.0 - t) * (1.0 - t)
                + 2.0 * self.ctrl0.x * (1.0 - t) * t
                + self.end.x * t * t,

            y: self.start.y * (1.0 - t) * (1.0 - t)
                + 2.0 * self.ctrl0.y * (1.0 - t) * t
                + self.end.y * t * t,
        }
    }

    fn derivative(&self) -> Self::Derivative {
        Line {
            start: 2.0 * (self.ctrl0 - self.start),
            end: 2.0 * (self.end - self.ctrl0),
        }
    }
}

#[cfg(test)]
mod bezier2_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::bezier::Bezier2;
    use crate::bezier::Curve;
    use crate::math::Vector;

    const B: Bezier2 = Bezier2 {
        start: Vector { x: 0.0, y: 0.0 },
        ctrl0: Vector { x: 0.75, y: 0.25 },
        end: Vector { x: 1.0, y: 1.0 },
    };

    #[test]
    fn start_is_t0() {
        assert_close2(B.at(0.0), Vector { x: 0.0, y: 0.0 });
    }

    #[test]
    fn end_is_t1() {
        assert_close2(B.at(1.0), Vector { x: 1.0, y: 1.0 });
    }

    #[test]
    fn mid() {
        assert_close2(B.at(0.5), Vector { x: 0.625, y: 0.375 });
    }

    #[test]
    fn derivative() {
        let d = B.derivative();
        assert_close2(d.start, Vector { x: 1.5, y: 0.5 });
        assert_close2(d.end, Vector { x: 0.5, y: 1.5 });
    }

    #[test]
    fn closest_point() {
        let (t, p) = B.closest_point(Vector { x: 0.75, y: 0.25 });
        assert_close(t, 0.5);
        assert_close2(p, Vector { x: 0.625, y: 0.375 });
    }

    #[test]
    fn start_curvature() {
        assert_close(B.curvature(0.0), 0.50596446);
    }

    #[test]
    fn mid_curvature() {
        assert_close(B.curvature(0.5), core::f32::consts::FRAC_1_SQRT_2);
    }

    #[test]
    fn end_curvature() {
        assert_close(B.curvature(1.0), 0.50596446);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bezier3 {
    pub start: Vector,
    pub ctrl0: Vector,
    pub ctrl1: Vector,
    pub end: Vector,
}

impl Curve for Bezier3 {
    type Derivative = Bezier2;

    /// Evaluate the curve at `t`
    fn at(&self, t: f32) -> Vector {
        Vector {
            x: self.start.x * (1.0 - t) * (1.0 - t) * (1.0 - t)
                + 3.0 * self.ctrl0.x * (1.0 - t) * (1.0 - t) * t
                + 3.0 * self.ctrl1.x * (1.0 - t) * t * t
                + self.end.x * t * t * t,

            y: self.start.y * (1.0 - t) * (1.0 - t) * (1.0 - t)
                + 3.0 * self.ctrl0.y * (1.0 - t) * (1.0 - t) * t
                + 3.0 * self.ctrl1.y * (1.0 - t) * t * t
                + self.end.y * t * t * t,
        }
    }

    fn derivative(&self) -> Self::Derivative {
        Bezier2 {
            start: 3.0 * (self.ctrl0 - self.start),
            ctrl0: 3.0 * (self.ctrl1 - self.ctrl0),
            end: 3.0 * (self.end - self.ctrl1),
        }
    }
}

#[cfg(test)]
mod bezier3_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::bezier::Bezier3;
    use crate::bezier::Curve;
    use crate::math::Vector;

    const B: Bezier3 = Bezier3 {
        start: Vector { x: 0.0, y: 0.0 },
        ctrl0: Vector { x: 0.5, y: 0.0 },
        ctrl1: Vector { x: 1.0, y: 0.5 },
        end: Vector { x: 1.0, y: 1.0 },
    };

    #[test]
    fn start_is_t0() {
        assert_close2(B.at(0.0), Vector { x: 0.0, y: 0.0 });
    }

    #[test]
    fn end_is_t1() {
        assert_close2(B.at(1.0), Vector { x: 1.0, y: 1.0 });
    }

    #[test]
    fn mid() {
        assert_close2(
            B.at(0.5),
            Vector {
                x: 0.6875,
                y: 0.3125,
            },
        );
    }

    #[test]
    fn derivative() {
        let d = B.derivative();
        assert_close2(d.start, Vector { x: 1.5, y: 0.0 });
        assert_close2(d.ctrl0, Vector { x: 1.5, y: 1.5 });
        assert_close2(d.end, Vector { x: 0.0, y: 1.5 });
    }

    #[test]
    fn closest_point_outside() {
        let (t, p) = B.closest_point(Vector { x: 0.75, y: 0.25 });
        assert_close(t, 0.5);
        assert_close2(
            p,
            Vector {
                x: 0.6875,
                y: 0.3125,
            },
        );
    }

    #[test]
    fn closest_point_inside() {
        let (t, p) = B.closest_point(Vector { x: 0.5, y: 0.5 });
        assert_close(t, 0.5);
        assert_close2(
            p,
            Vector {
                x: 0.6875,
                y: 0.3125,
            },
        );
    }

    #[test]
    fn closest_point_before_outside() {
        let (t, p) = B.closest_point(Vector { x: -1.0, y: -0.25 });
        assert!(t < 0.0);
        assert_close2(p, Vector { x: -1.0, y: 0.0 });
    }

    #[test]
    fn closest_point_before_inside() {
        let (t, p) = B.closest_point(Vector { x: -1.0, y: 0.25 });
        assert!(t < 0.0);
        assert_close2(p, Vector { x: -1.0, y: 0.0 });
    }

    #[test]
    fn closest_point_after_outside() {
        let (t, p) = B.closest_point(Vector { x: 1.25, y: 2.0 });
        assert!(t > 1.0);
        assert_close2(p, Vector { x: 1.0, y: 2.0 });
    }

    #[test]
    fn closest_point_after_inside() {
        let (t, p) = B.closest_point(Vector { x: 0.75, y: 2.0 });
        assert!(t > 1.0);
        assert_close2(p, Vector { x: 1.0, y: 2.0 });
    }

    // Observed in simulator when first testing
    #[test]
    fn closest_point_after_from_sim() {
        let b = Bezier3 {
            start: Vector {
                x: 1260.0,
                y: 1170.0,
            },
            ctrl0: Vector {
                x: 1440.0,
                y: 1170.0,
            },
            ctrl1: Vector {
                x: 1440.0,
                y: 1170.0,
            },
            end: Vector {
                x: 1620.0,
                y: 1170.0,
            },
        };
        let (t, p) = b.closest_point(Vector {
            x: 1861.0,
            y: 1170.0,
        });
        assert!(t > 1.0);
        assert_close2(
            p,
            Vector {
                x: 1861.0,
                y: 1170.0,
            },
        )
    }

    #[test]
    fn start_curvature() {
        assert_close(B.curvature(0.0), 1.333333);
    }

    #[test]
    fn mid_curvature() {
        assert_close(B.curvature(0.5), 0.8380524);
    }

    #[test]
    fn end_curvature() {
        assert_close(B.curvature(1.0), 1.3333333);
    }
}
