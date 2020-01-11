/*!
 *  Algorithms to follow a path
 *
 *  A Segment is just one part of a larger path. These can be fed to a Path to follow one
 */

use core::f32::consts::FRAC_PI_2;
use core::f32::consts::PI;

use serde::Deserialize;
use serde::Serialize;

use libm::F32Ext;

use heapless::consts::U16;
use heapless::Vec;
use typenum::Unsigned;

use vek::CubicBezier2;

use pid_control::Controller;
use pid_control::DerivativeMode;
use pid_control::PIDController;

use crate::map::Direction;
use crate::map::Orientation;
use crate::map::Vector;
use crate::map::DIRECTION_PI_2;

/// Generates a circle path
///
/// The `start` is the bottom of the circle
pub fn circle(start: Vector, center: Vector) -> [Segment; 2] {
    [
        Segment::Arc(start, center, PI),
        Segment::Arc(start + 2.0 * (center - start), center, PI),
    ]
}

/// Generates an 'oval' path, consisting of two semicircles connected by lines
pub fn oval(start: Vector, width: f32, height: f32) -> [Segment; 6] {
    let radius = height / 2.0;
    [
        Segment::Arc(
            Vector {
                x: start.x,
                y: start.y + radius,
            },
            Vector {
                x: start.x + radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Arc(
            Vector {
                x: start.x + radius,
                y: start.y + height,
            },
            Vector {
                x: start.x + radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x + width - radius,
                y: start.y + height,
            },
            Vector {
                x: start.x + radius,
                y: start.y + height,
            },
        ),
        Segment::Arc(
            Vector {
                x: start.x + width,
                y: start.y + radius,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Arc(
            Vector {
                x: start.x + width - radius,
                y: start.y,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x + radius,
                y: start.y,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y,
            },
        ),
    ]
}

/// Generates a rectangle with rounded corners.
///
/// Note that if the radius is zero, there will be zero-length arcs at the corners,
/// and if the radius is half the width or height, there will be zero-length lines.
/// These will cause NaNs when trying to follow them and should be avoided
///
/// TODO: Don't generate zero-length segments
pub fn rounded_rectangle(
    start: Vector,
    width: f32,
    height: f32,
    radius: f32,
) -> [Segment; 8] {
    [
        Segment::Arc(
            Vector {
                x: start.x,
                y: start.y + radius,
            },
            Vector {
                x: start.x + radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x,
                y: start.y + height - radius,
            },
            Vector {
                x: start.x,
                y: start.y + radius,
            },
        ),
        Segment::Arc(
            Vector {
                x: start.x + radius,
                y: start.y + height,
            },
            Vector {
                x: start.x + radius,
                y: start.y + height - radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x + width - radius,
                y: start.y + height,
            },
            Vector {
                x: start.x + radius,
                y: start.y + height,
            },
        ),
        Segment::Arc(
            Vector {
                x: start.x + width,
                y: start.y + height - radius,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y + height - radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x + width,
                y: start.y + radius,
            },
            Vector {
                x: start.x + width,
                y: start.y + height - radius,
            },
        ),
        Segment::Arc(
            Vector {
                x: start.x + width - radius,
                y: start.y,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y + radius,
            },
            FRAC_PI_2,
        ),
        Segment::Line(
            Vector {
                x: start.x + radius,
                y: start.y,
            },
            Vector {
                x: start.x + width - radius,
                y: start.y,
            },
        ),
    ]
}

/// Generate a bezier for a corner
///
/// # Arguments
///
/// `center`: the location of the corner, where the two lines intersect
///
/// `start`: the absolute direction of the entrance line
///
/// `end`: the absolute direction of the exit line
///
/// `radius` is the distance from the center to the end of each line
pub fn bezier_corner(
    center: Vector,
    start: Direction,
    end: Direction,
    radius: f32,
) -> Segment {
    Segment::Bezier(
        center - radius * start.into_unit_vector(),
        center,
        center,
        center + radius * end.into_unit_vector(),
    )
}

#[cfg(test)]
mod segment_generation_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::bezier_corner;
    use super::Segment;
    use crate::map::Direction;
    use crate::map::Vector;
    use core::f32::consts::FRAC_PI_2;

    #[test]
    fn bezier_corner_test() {
        let segment = bezier_corner(
            Vector { x: 90.0, y: 0.0 },
            Direction::from(0.0),
            Direction::from(FRAC_PI_2),
            90.0,
        );

        if let Segment::Bezier(s, c0, c1, e) = segment {
            assert_close2(s, Vector { x: 0.0, y: 0.0 });
            assert_close2(c0, Vector { x: 90.0, y: 0.0 });
            assert_close2(c1, Vector { x: 90.0, y: 0.0 });
            assert_close2(e, Vector { x: 90.0, y: 90.0 });
        } else {
            panic!("Not a bezier!");
        }
    }
}

/// How accurate to be when searching on a bezier for the closest point
///
/// I have no idea what a good number is
const BEZIER_SEARCH_ELIPSON: f32 = 0.0000001;

/// The number of steps to use when broad searching on a bezier for the closest point
///
/// I have no idea what a good number is
const BEZIER_SEARCH_STEPS: u16 = 32;

/**
 * A segment of a larger path
 *
 * The path following algorithm uses the distance from the path to control steering of the mouse,
 * and the distance along it with the total distance to determine when the segment is complete.
 * The distance along may also be used to control forward velocity
 *
 * Usually, the segments are arranged so that each one starts at the end of the previous one and
 * are tangent. This makes the movement nice and smooth. However, it does not have to be for eg.
 * turning around in place.
 */
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Segment {
    /**
     * A line segment is defined by start and end points.
     *
     * See also [Desmos](https://www.desmos.com/calculator/yve8exartj)
     */
    Line(Vector, Vector),

    /**
     * An arc is defined by a starting point, center point and an direction in radians
     *
     * See also [Desmos](https://www.desmos.com/calculator/4dcrt6qz4p)
     */
    Arc(Vector, Vector, f32),

    /**
     * A cubic bezier curve defined by a start point, two control points, and an end point.
     * This allows careful control of the curvature of the curve (1/r, closely related to the
     * angular ratio used for motion) to keep it within mechanical limits
     *
     * See also [Desmos](https://www.desmos.com/calculator/xfthfifnwp)
     *
     * ## References
     * [https://pomax.github.io/bezierinfo/] is a good into to beziers
     *
     * https://www.hindawi.com/journals/jat/2018/6060924/ describes using higher order beziers for
     * motion control to keep curvature under control.
     *
     * https://www.cl.cam.ac.uk/teaching/2000/AGraphHCI/SMEG/node3.html shows finding the points to
     * keep the curvature of two adjacent curves the same
     */
    Bezier(Vector, Vector, Vector, Vector),
}

impl Segment {
    /// Tests whether the segment of the path is completed at the given point
    ///
    /// This is often faster than computing the distance along the path and comparing with the
    /// total length
    pub fn complete(&self, m: Vector) -> bool {
        match self {
            &Segment::Line(l1, l2) => {
                let mouse = m - l1;
                let line = l2 - l1;

                let i = mouse.project_onto(line) + l1;

                let distance_along = (i - l1).magnitude();
                let total_distance = (l1 - l2).magnitude();

                distance_along >= total_distance
            }
            &Segment::Arc(s, c, t) => {
                let v_mouse = m - c;
                let v_start = s - c;

                let r_mouse = v_mouse.magnitude();
                let r_start = v_start.magnitude();

                let angle_along = F32Ext::acos(
                    (v_mouse.x * v_start.x + v_mouse.y * v_start.y)
                        / (r_mouse * r_start),
                );

                angle_along >= t
            }
            &Segment::Bezier(s, c0, c1, e) => {
                let bezier = CubicBezier2 {
                    start: s.into(),
                    ctrl0: c0.into(),
                    ctrl1: c1.into(),
                    end: e.into(),
                };

                let (t, _point) = bezier.binary_search_point_by_steps(
                    m.into(),
                    BEZIER_SEARCH_STEPS,
                    BEZIER_SEARCH_ELIPSON,
                );

                t >= 1.0
            }
        }
    }

    /// Find the distance from point `m` to the closest point on the path
    /// If `m` is to the right, the distance should be negative, if `m` is to the left, the distance
    /// should be positive
    pub fn distance_from(&self, m: Vector) -> f32 {
        match self {
            &Segment::Line(l1, l2) => {
                let mouse = m - l1;
                let line = l2 - l1;

                let i = mouse.project_onto(line) + l1;

                if line.cross(mouse) > 0.0 {
                    (i - m).magnitude()
                } else {
                    -(i - m).magnitude()
                }
            }
            &Segment::Arc(s, c, t) => {
                let v_mouse = m - c;
                let v_start = s - c;

                if t > 0.0 {
                    v_start.magnitude() - v_mouse.magnitude()
                } else {
                    v_mouse.magnitude() - v_start.magnitude()
                }
            }
            &Segment::Bezier(s, c0, c1, e) => {
                let bezier = CubicBezier2 {
                    start: s.into(),
                    ctrl0: c0.into(),
                    ctrl1: c1.into(),
                    end: e.into(),
                };

                let (t, point) = bezier.binary_search_point_by_steps(
                    m.into(),
                    BEZIER_SEARCH_STEPS,
                    BEZIER_SEARCH_ELIPSON,
                );

                let p: Vector = point.into();

                let mouse = m - p;
                let tangent = Vector::from(bezier.evaluate_derivative(t));

                if tangent.cross(mouse) > 0.0 {
                    (p - m).magnitude()
                } else {
                    -(p - m).magnitude()
                }
            }
        }
    }

    /// Find the direction tangent to the path at the point on the path closest to point `m`
    pub fn tangent_direction(&self, m: Vector) -> Direction {
        match self {
            &Segment::Line(l1, l2) => {
                let v_line = l2 - l1;
                Direction::from(F32Ext::atan2(v_line.y, v_line.x))
            }
            &Segment::Arc(_s, c, t) => {
                let v_mouse = m - c;

                let perpendicular_direction = v_mouse.direction();

                if t >= 0.0 {
                    perpendicular_direction + DIRECTION_PI_2
                } else {
                    perpendicular_direction - DIRECTION_PI_2
                }
            }
            &Segment::Bezier(s, c0, c1, e) => {
                let bezier = CubicBezier2 {
                    start: s.into(),
                    ctrl0: c0.into(),
                    ctrl1: c1.into(),
                    end: e.into(),
                };

                let (t, _p) = bezier.binary_search_point_by_steps(
                    m.into(),
                    BEZIER_SEARCH_STEPS,
                    BEZIER_SEARCH_ELIPSON,
                );

                Vector::from(bezier.evaluate_derivative(t)).direction()
            }
        }
    }
}

#[cfg(test)]
mod segment_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use core::f32::consts::FRAC_PI_4;
    use core::f32::consts::SQRT_2;

    use super::Segment;
    use super::Vector;
    use crate::path::Segment::Bezier;

    const MAX_DELTA: f32 = 0.000001;

    const MOUSE: Vector = Vector { x: 5.0, y: 3.0 };
    const MOUSE2: Vector = Vector { x: 3.0, y: 5.0 };

    const LINE_SEGMENT: Segment =
        Segment::Line(Vector { x: 2.0, y: 2.0 }, Vector { x: 6.0, y: 6.0 });

    #[test]
    fn segment_line_complete() {
        assert!(!LINE_SEGMENT.complete(Vector { x: 4.0, y: 5.0 }));
    }
    #[test]
    fn segment_line_complete2() {
        assert!(LINE_SEGMENT.complete(Vector { x: 7.0, y: 8.0 }));
    }
    #[test]
    fn segment_line_distance_from() {
        assert_close(LINE_SEGMENT.distance_from(MOUSE), -SQRT_2);
    }
    #[test]
    fn segment_line_distance_from2() {
        assert_close(LINE_SEGMENT.distance_from(MOUSE2), SQRT_2);
    }
    #[test]
    fn segment_line_tangent_angle() {
        assert_close(
            f32::from(LINE_SEGMENT.tangent_direction(MOUSE)),
            FRAC_PI_4,
        );
    }

    const BEZIER_SEGMENT: Segment = Segment::Bezier(
        Vector { x: 0.0, y: 0.0 },
        Vector { x: 4.0, y: 0.0 },
        Vector { x: 4.0, y: 0.0 },
        Vector { x: 4.0, y: 4.0 },
    );

    #[test]
    fn bezier_complete() {
        assert!(!BEZIER_SEGMENT.complete(Vector { x: 3.75, y: 0.25 }));
    }

    #[test]
    fn bezier_complete2() {
        assert!(BEZIER_SEGMENT.complete(Vector { x: 3.0, y: 5.0 }));
    }

    // The bezier happens to pass through (3.5, 0.5)
    // or at least close enough for the tests to pass

    #[test]
    fn bezier_distance_from() {
        assert_close(
            BEZIER_SEGMENT.distance_from(Vector { x: 3.75, y: 0.25 }),
            -SQRT_2 / 4.0,
        );
    }

    #[test]
    fn bezier_distance_from2() {
        assert_close(
            BEZIER_SEGMENT.distance_from(Vector { x: 3.0, y: 1.0 }),
            SQRT_2 / 2.0,
        )
    }

    #[test]
    fn bezier_tangent_direction() {
        assert_close(
            f32::from(
                BEZIER_SEGMENT.tangent_direction(Vector { x: 3.75, y: 0.25 }),
            ),
            FRAC_PI_4,
        );
    }
}

pub type PathBufLen = U16;
pub type PathBuf = Vec<Segment, PathBufLen>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathDebug {
    pub path: Option<PathBuf>,
    pub distance_from: Option<f32>,
    pub centered_direction: Option<f32>,
    pub tangent_direction: Option<Direction>,
    pub target_direction: Option<Direction>,
    pub target_direction_offset: Option<f32>,
    pub error: Option<f32>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub offset_p: f32,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub pid: PIDController,
    pub segment_buffer: PathBuf,
    pub time: u32,
}

impl Path {
    pub fn new(config: &PathConfig, time: u32) -> Path {
        let mut pid = PIDController::new(
            config.p as f64,
            config.i as f64,
            config.d as f64,
        );
        pid.d_mode = DerivativeMode::OnError;
        //pid.set_limits(-1.0, 1.0);
        Path {
            pid,
            segment_buffer: Vec::new(),
            time,
        }
    }

    pub fn add_segments(
        &mut self,
        segments: &[Segment],
    ) -> Result<usize, usize> {
        for (i, segment) in segments.iter().enumerate() {
            if self.segment_buffer.push(*segment).is_err() {
                return Err(i);
            }
        }

        Ok(PathBufLen::to_usize() - self.segment_buffer.len())
    }

    pub fn update(
        &mut self,
        config: &PathConfig,
        time: u32,
        orientation: Orientation,
    ) -> (f32, bool, PathDebug) {
        let mut debug = PathDebug {
            path: None,
            distance_from: None,
            centered_direction: None,
            tangent_direction: None,
            target_direction: None,
            target_direction_offset: None,
            error: None,
        };

        self.pid.p_gain = config.p as f64;
        self.pid.i_gain = config.i as f64;
        self.pid.d_gain = config.d as f64;

        let delta_time = time - self.time;

        // Check if we are done with the current segment
        if let Some(segment) = self.segment_buffer.last() {
            if segment.complete(orientation.position) {
                self.segment_buffer.pop();
            }
        }

        // Calculate a target direction based on the distance from the path
        let (target_direction, done) = if let Some(segment) =
            self.segment_buffer.last()
        {
            let offset = segment.distance_from(orientation.position);
            let tangent_direction =
                segment.tangent_direction(orientation.position);

            // This s-curve will asymptote at -pi/2 and pi/2, and cross the origin.
            // Points the mouse directly at the path far away, but along the path
            // close up. The offset_p determines how aggressive it is
            let target_direction_offset =
                PI / (1.0 + F32Ext::exp(config.offset_p * offset)) - FRAC_PI_2;

            let target_direction =
                tangent_direction + Direction::from(target_direction_offset);

            debug.distance_from = Some(offset);
            debug.tangent_direction = Some(tangent_direction);
            debug.target_direction = Some(target_direction);
            debug.target_direction_offset = Some(target_direction_offset);

            (target_direction, false)
        } else {
            (Direction::from(0.0), true)
        };

        // Do PID on the current and target directions
        let centered_direction =
            orientation.direction.centered_at(target_direction);

        self.pid.set_target(target_direction.into());
        let angular_power = self
            .pid
            .update(centered_direction as f64, delta_time as f64)
            as f32;

        debug.path = Some(self.segment_buffer.clone());
        debug.centered_direction = Some(centered_direction);
        debug.error = Some(f32::from(target_direction) - centered_direction);

        self.time = time;

        (angular_power, done, debug)
    }
}
