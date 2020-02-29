use heapless::Vec;
use typenum::{Unsigned, U16};

use serde::{Deserialize, Serialize};

use crate::fast::path::PathMotion;
use crate::fast::turn::TurnMotion;
use crate::fast::Orientation;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Motion {
    Path(PathMotion),
    Turn(TurnMotion),
}

impl Motion {
    pub fn done(&self, orientation: Orientation) -> bool {
        match self {
            Motion::Path(path_motion) => path_motion.done(orientation),
            Motion::Turn(turn_motion) => turn_motion.done(orientation),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionQueueDebug {
    queue: MotionQueueBuffer,
}

pub type MotionQueueSize = U16;
pub type MotionQueueBuffer = Vec<Motion, MotionQueueSize>;

pub struct MotionQueue {
    queue: MotionQueueBuffer,
}

impl MotionQueue {
    pub fn new() -> MotionQueue {
        MotionQueue { queue: Vec::new() }
    }

    pub fn add_motions(&mut self, motions: &[Motion]) -> Result<usize, usize> {
        for (i, motion) in motions.iter().enumerate() {
            if self.queue.push(*motion).is_err() {
                return Err(i);
            }
        }

        Ok(MotionQueueSize::to_usize() - self.queue.len())
    }

    pub fn pop_completed(&mut self, orientation: Orientation) -> MotionQueueDebug {
        // Go through the buffer and pop off any moves that have been completed
        while let Some(motion) = self.queue.pop() {
            if !motion.done(orientation) {
                self.queue.push(motion).ok();
                break;
            }
        }

        MotionQueueDebug {
            queue: self.queue.clone(),
        }
    }

    pub fn motions_remaining(&self) -> usize {
        self.queue.len()
    }

    pub fn next_motion(&self) -> Option<Motion> {
        self.queue.first().cloned()
    }
}
