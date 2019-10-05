use arrayvec::ArrayVec;

use pid_control::Controller;
use pid_control::DerivativeMode;
use pid_control::PIDController;

use crate::map::Vector;
use core::borrow::Borrow;

#[derive(Copy, Clone, Debug)]
pub enum Segment {
    Line(Vector, Vector),
}

impl Segment {
    pub fn total_distance(&self) -> f32 {
        match self {
            &Segment::Line(l1, l2) => (l1 - l2).magnitude(),
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
        assert_close(LINE_SEGMENT.distance_from(MOUSE), 1.41421356237);
    }
    #[test]
    fn segment_line_distance_from2() {
        assert_close(LINE_SEGMENT.distance_from(MOUSE2), -1.41421356237);
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

pub const PATH_BUF_LEN: usize = 64;

#[derive(Debug)]
pub struct PathDebug<'a> {
    pub path: Option<&'a [Segment]>,
    pub distance_from: Option<f32>,
    pub distance_along: Option<f32>,
}

#[derive(Debug)]
pub struct PathConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub pid: PIDController,
    pub segment_buffer: ArrayVec<[Segment; PATH_BUF_LEN]>,
    pub time: u32,
}

impl Path {
    pub fn new(config: &PathConfig, time: u32) -> Path {
        let mut pid = PIDController::new(config.p as f64, config.i as f64, config.d as f64);
        pid.d_mode = DerivativeMode::OnError;
        pid.set_limits(-1.0, 1.0);
        Path {
            pid,
            segment_buffer: ArrayVec::new(),
            time,
        }
    }

    pub fn add_segments(&mut self, segments: &[Segment]) -> Result<usize, usize> {
        for (i, segment) in segments.iter().enumerate() {
            if self.segment_buffer.try_push(*segment).is_err() {
                return Err(i);
            }
        }

        Ok(PATH_BUF_LEN - self.segment_buffer.len())
    }

    pub fn update(
        &mut self,
        config: &PathConfig,
        time: u32,
        position: Vector,
    ) -> (f32, bool, PathDebug) {
        let mut debug = PathDebug {
            path: None,
            distance_from: None,
            distance_along: None,
        };

        self.pid.p_gain = config.p as f64;
        self.pid.i_gain = config.i as f64;
        self.pid.d_gain = config.d as f64;

        let delta_time = time - self.time;

        // Check if we are done with the current segment
        if let Some(segment) = self.segment_buffer.last() {
            if segment.distance_along(position) >= segment.total_distance() {
                self.segment_buffer.pop();
            }
        }

        // Do pid on the distance from the path
        let (angular_power, done) = if let Some(segment) = self.segment_buffer.last() {
            let offset = segment.distance_from(position);
            debug.distance_from = Some(offset);
            debug.distance_along = Some(segment.distance_along(position));
            (
                self.pid.update(offset as f64, delta_time as f64) as f32,
                false,
            )
        } else {
            (0.0, true)
        };

        debug.path = Some(self.segment_buffer.as_ref());

        (angular_power, done, debug)
    }
}
