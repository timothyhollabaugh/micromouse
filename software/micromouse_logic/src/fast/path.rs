/*!
 *  Algorithms to follow a path
 *
 *  A Segment is just one part of a larger path. These can be fed to a Path to follow one
 */

use core::f32::consts::FRAC_PI_2;

use serde::{Deserialize, Serialize};

use pid_control::{Controller, PIDController};

use super::{Direction, Orientation, Vector};

use super::curve::{Bezier5, Curve};
use crate::config::MechanicalConfig;

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
pub struct PathMotion {
    bezier: Bezier5,
}

impl PathMotion {
    /// Generate a corner
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
    pub fn corner(
        center: Vector,
        start: Direction,
        end: Direction,
        radius: f32,
    ) -> PathMotion {
        let start_v = start.into_unit_vector();
        let end_v = end.into_unit_vector();
        PathMotion {
            bezier: Bezier5 {
                start: center - radius * start_v,
                ctrl0: center - (radius / 2.0) * start_v,
                ctrl1: center - (radius / 3.0) * start_v,
                ctrl2: center + (radius / 3.0) * end_v,
                ctrl3: center + (radius / 2.0) * end_v,
                end: center + radius * end_v,
            },
        }
    }

    /// Generate a strait line
    pub fn line(start: Vector, end: Vector) -> PathMotion {
        let mid = (end - start) * 0.5 + start;
        PathMotion {
            bezier: Bezier5 {
                start,
                ctrl0: mid,
                ctrl1: mid,
                ctrl2: mid,
                ctrl3: mid,
                end,
            },
        }
    }

    /// Find the point on the segment closest to `m`
    pub fn closest_point(&self, m: Vector) -> (f32, Vector) {
        self.bezier.closest_point(m)
    }

    /// Derivative at `t`
    pub fn derivative(&self, t: f32) -> Vector {
        self.bezier.derivative().at(t)
    }

    /// Curvature at `t`
    pub fn curvature(&self, t: f32) -> f32 {
        self.bezier.curvature(t)
    }

    /// Whether this path motion is done or not
    pub fn done(&self, orientation: Orientation) -> bool {
        self.bezier.closest_point(orientation.position).0 >= 1.0
    }
}

// Adjust the curvature for the mouse not being on the path
fn offset_curvature(curvature: f32, distance: f32) -> f32 {
    let r = 1.0 / curvature;

    let r2 = if curvature > 0.0 {
        r - distance
    } else {
        r + distance
    };

    let curvature2 = 1.0 / r2;
    curvature2
}

#[cfg(test)]
mod offset_curvature_tests {
    use super::offset_curvature;
    #[allow(unused_imports)]
    use crate::test::*;

    #[test]
    fn zero_distance_positive_curvature() {
        assert_close(offset_curvature(1.0, 0.0), 1.0)
    }

    #[test]
    fn positive_distance_positive_curvature() {
        assert_close(offset_curvature(1.0, 0.5), 2.0)
    }

    #[test]
    fn negative_distance_positive_curvature() {
        assert_close(offset_curvature(1.0, -0.5), 0.6666667)
    }

    #[test]
    fn zero_distance_negative_curvature() {
        assert_close(offset_curvature(-1.0, 0.0), -1.0)
    }

    #[test]
    fn positive_distance_negative_curvature() {
        assert_close(offset_curvature(-1.0, 0.5), -2.0)
    }

    #[test]
    fn negative_distance_negative_curvature() {
        assert_close(offset_curvature(-1.0, -0.5), -0.66666667)
    }

    #[test]
    fn zero_curvature() {
        assert_close(offset_curvature(0.0, 0.5), 0.0)
    }
}

fn curvature_to_left_right(
    config: &MechanicalConfig,
    velocity: f32,
    curvature: f32,
) -> (f32, f32) {
    let rotations_per_ms = velocity * curvature;
    let angular_mm_per_ms = rotations_per_ms * config.wheelbase / 2.0;
    let left = velocity - angular_mm_per_ms;
    let right = velocity + angular_mm_per_ms;
    (left, right)
}

#[cfg(test)]
mod curvature_to_left_right_test {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::curvature_to_left_right;
    use crate::config::MechanicalConfig;

    const CONFIG: MechanicalConfig = crate::config::mouse_2019::MECH;

    #[test]
    fn test_curvature_to_left_right_circle() {
        let (left, right) = curvature_to_left_right(&CONFIG, 0.5, 1.0 / 90.0);
        assert_close(left, 0.294444);
        assert_close(right, 0.705556);
    }

    #[test]
    fn test_curvature_to_left_right_straight() {
        let (left, right) = curvature_to_left_right(&CONFIG, 0.5, 0.0);
        assert_close(left, 0.5);
        assert_close(right, 0.5);
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathHandlerDebug {
    pub closest_point: Option<(f32, Vector)>,
    pub distance_from: Option<f32>,
    pub tangent_direction: Option<Direction>,
    pub adjust_direction: Option<Direction>,
    pub centered_direction: Option<f32>,
    pub adjust_curvature: Option<f32>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathHandlerConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub offset_p: f32,
    pub velocity: f32,
}

#[derive(Clone, Debug)]
pub struct PathHandler {
    pub direction_pid: PIDController,
    pub time: u32,
}

impl PathHandler {
    pub fn new(config: &PathHandlerConfig, time: u32) -> PathHandler {
        let pid = PIDController::new(config.p as f64, config.i as f64, config.d as f64);
        PathHandler {
            direction_pid: pid,
            time,
        }
    }

    pub fn update(
        &mut self,
        config: &PathHandlerConfig,
        mech: &MechanicalConfig,
        time: u32,
        orientation: Orientation,
        segment: PathMotion,
    ) -> (f32, f32, Direction, PathHandlerDebug) {
        let mut debug = PathHandlerDebug::default();

        let delta_time = time - self.time;

        self.direction_pid.p_gain = config.p as f64;
        self.direction_pid.i_gain = config.i as f64;
        self.direction_pid.d_gain = config.d as f64;

        // Gather some info about the segment
        let (t, p) = segment.closest_point(orientation.position);
        let v_tangent = segment.derivative(t);
        let v_m = orientation.position - p;
        let distance = if v_tangent.cross(v_m) > 0.0 {
            v_m.magnitude()
        } else {
            -v_m.magnitude()
        };

        let tangent = v_tangent.direction();

        let path_curvature = segment.curvature(t);

        // If there was another segment, try to follow it
        // The curvature of the path where the mouse is
        let offset_curvature = offset_curvature(path_curvature, distance);

        let adjust_curvature = if config.offset_p != 0.0 {
            // Need to calculate an adjustment curvature to get the mouse back on the path
            // This gets added to the offset curvature above to get the final path curvature.
            // As such, it should always turn the mouse towards the path, but avoid turning
            // past the path. This is done by calculating a target direction that points towards the
            // path far away, but along the path close up. A curvature is then calculated that
            // should get the mouse to that direction in the next loop (assuming no physics
            // limitations. This should probably be limited base on the mechanics).

            // This s-curve will asymptote at -pi/2 and pi/2, and cross the origin.
            // Points the mouse directly at the path far away, but along the path
            // close up. The offset_p determines how aggressive it is
            let adjust_direction_offset = -distance * config.offset_p;

            let adjust_direction_offset = if adjust_direction_offset > FRAC_PI_2 {
                FRAC_PI_2
            } else if adjust_direction_offset < -FRAC_PI_2 {
                -FRAC_PI_2
            } else {
                adjust_direction_offset
            };

            //PI / (1.0 + F32Ext::exp(config.offset_p * distance)) - FRAC_PI_2;

            let adjust_direction = tangent + Direction::from(adjust_direction_offset);
            debug.adjust_direction = Some(adjust_direction);

            let centered_direction = orientation.direction.centered_at(adjust_direction);
            debug.centered_direction = Some(centered_direction);

            self.direction_pid
                .set_target(f32::from(adjust_direction) as f64);
            self.direction_pid
                .update(centered_direction as f64, delta_time as f64) as f32
        } else {
            0.0
        };

        let target_curvature = offset_curvature + adjust_curvature;

        let (target_left_velocity, target_right_velocity) =
            curvature_to_left_right(mech, config.velocity, target_curvature);

        debug.distance_from = Some(distance);
        debug.tangent_direction = Some(tangent);
        debug.adjust_curvature = Some(adjust_curvature);

        self.time = time;

        (target_left_velocity, target_right_velocity, tangent, debug)
    }
}
