use core::f32::consts::FRAC_PI_2;
use core::f32::consts::PI;

use serde::Deserialize;
use serde::Serialize;

use libm::F32Ext;

use heapless::consts::U16;
use heapless::Vec;
use typenum::Unsigned;

use pid_control::Controller;
use pid_control::DerivativeMode;
use pid_control::PIDController;

use crate::map::Direction;
use crate::map::Orientation;
use crate::map::Vector;
use crate::map::DIRECTION_PI_2;

pub fn circle(start: Vector, center: Vector) -> [Segment; 2] {
    [
        Segment::Arc(start, center, PI),
        Segment::Arc(start + 2.0 * (center - start), center, PI),
    ]
}

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

pub fn rounded_rectangle(start: Vector, width: f32, height: f32, radius: f32) -> [Segment; 8] {
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
     * See https://www.desmos.com/calculator/yve8exartj
     */
    Line(Vector, Vector),

    /**
     * An arc is defined by a starting point, center point and an direction in radians
     * See https://www.desmos.com/calculator/4dcrt6qz4p
     */
    Arc(Vector, Vector, f32),
}

impl Segment {
    pub fn total_distance(&self) -> f32 {
        match self {
            &Segment::Line(l1, l2) => (l1 - l2).magnitude(),
            &Segment::Arc(s, c, t) => F32Ext::abs(t) * (s - c).magnitude(),
        }
    }

    pub fn distance_along(&self, m: Vector) -> f32 {
        match self {
            &Segment::Line(l1, l2) => {
                let mouse = m - l1;
                let line = l2 - l1;

                let i = Vector {
                    x: (mouse.x * line.x * line.x + mouse.y * line.y * line.x)
                        / (line.x * line.x + line.y * line.y)
                        + l1.x,
                    y: (mouse.x * line.x * line.y + mouse.y * line.y * line.y)
                        / (line.x * line.x + line.y * line.y)
                        + l1.y,
                };

                (i - l1).magnitude()
            }

            &Segment::Arc(s, c, _t) => {
                let v_mouse = m - c;
                let v_start = s - c;

                let r_mouse = v_mouse.magnitude();
                let r_start = v_start.magnitude();

                r_start
                    * F32Ext::acos(
                        (v_mouse.x * v_start.x + v_mouse.y * v_start.y) / (r_mouse * r_start),
                    )
            }
        }
    }

    pub fn distance_from(&self, m: Vector) -> f32 {
        match self {
            &Segment::Line(l1, l2) => {
                let mouse = m - l1;
                let line = l2 - l1;

                let i = Vector {
                    x: (mouse.x * line.x * line.x + mouse.y * line.y * line.x)
                        / (line.x * line.x + line.y * line.y)
                        + l1.x,
                    y: (mouse.x * line.x * line.y + mouse.y * line.y * line.y)
                        / (line.x * line.x + line.y * line.y)
                        + l1.y,
                };

                let cross_product = line.x * mouse.y - mouse.x * line.y;

                if cross_product > 0.0 {
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
        }
    }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Segment;
    use super::Vector;

    const MAX_DELTA: f32 = 0.000001;

    const LINE_SEGMENT: Segment =
        Segment::Line(Vector { x: 2.0, y: 2.0 }, Vector { x: 6.0, y: 6.0 });

    const MOUSE: Vector = Vector { x: 5.0, y: 3.0 };
    const MOUSE2: Vector = Vector { x: 3.0, y: 5.0 };

    #[test]
    fn segment_line_total_distance() {
        assert_close(LINE_SEGMENT.total_distance(), 5.65685424949);
    }
    #[test]
    fn segment_line_distance_along() {
        assert_close(LINE_SEGMENT.distance_along(MOUSE), 2.82842712475);
    }
    #[test]
    fn segment_line_distance_from() {
        assert_close(LINE_SEGMENT.distance_from(MOUSE), -1.41421356237);
    }
    #[test]
    fn segment_line_distance_from2() {
        assert_close(LINE_SEGMENT.distance_from(MOUSE2), 1.41421356237);
    }

    fn assert_close2(left: Vector, right: Vector) {
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

    fn assert_close(left: f32, right: f32) {
        let delta = (left - right).abs();
        assert!(
            delta <= MAX_DELTA,
            "\nleft: {}\nright: {}\ndelta: {}\n",
            left,
            right,
            delta
        );
    }
}

pub type PathBufLen = U16;
pub type PathBuf = Vec<Segment, PathBufLen>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathDebug {
    pub path: Option<PathBuf>,
    pub segment_length: Option<f32>,
    pub distance_from: Option<f32>,
    pub distance_along: Option<f32>,
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
        let mut pid = PIDController::new(config.p as f64, config.i as f64, config.d as f64);
        pid.d_mode = DerivativeMode::OnError;
        //pid.set_limits(-1.0, 1.0);
        Path {
            pid,
            segment_buffer: Vec::new(),
            time,
        }
    }

    pub fn add_segments(&mut self, segments: &[Segment]) -> Result<usize, usize> {
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
            segment_length: None,
            distance_from: None,
            distance_along: None,
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
            debug.segment_length = Some(segment.total_distance());
            if segment.distance_along(orientation.position) >= segment.total_distance() {
                self.segment_buffer.pop();
            }
        }

        // Do pid on the distance from the path
        let (target_direction, done) = if let Some(segment) = self.segment_buffer.last() {
            let offset = segment.distance_from(orientation.position);
            let tangent_direction = segment.tangent_direction(orientation.position);
            let target_direction_offset =
                PI / (1.0 + F32Ext::exp(config.offset_p * offset)) - FRAC_PI_2;
            let target_direction = tangent_direction + Direction::from(target_direction_offset);

            debug.distance_from = Some(offset);
            debug.distance_along = Some(segment.distance_along(orientation.position));
            debug.tangent_direction = Some(tangent_direction);
            debug.target_direction = Some(target_direction);
            debug.target_direction_offset = Some(target_direction_offset);

            (target_direction, false)
        } else {
            (Direction::from(0.0), true)
        };

        let centered_direction = orientation.direction.centered_at(target_direction);

        debug.centered_direction = Some(centered_direction);

        debug.error = Some(f32::from(target_direction) - centered_direction);

        self.pid.set_target(target_direction.into());
        let angular_power =
            self.pid
                .update(centered_direction as f64, delta_time as f64) as f32;

        debug.path = Some(self.segment_buffer.clone());

        self.time = time;

        (angular_power, done, debug)
    }
}
