use heapless::Vec;
use serde::Deserialize;
use serde::Serialize;
use typenum::consts::*;

use crate::fast::motor_control::MotorControlDebug;
use crate::fast::Orientation;

use crate::fast::localize::LocalizeDebug;
use crate::fast::motion_control::MotionHandlerDebug;
use crate::fast::motion_queue::MotionQueueDebug;
use crate::mouse::{HardwareDebug, MouseConfig};
use crate::slow::SlowDebug;

#[derive(Debug, Serialize, Deserialize)]
pub enum MouseMsg {
    NoOp,
    StopDebug,
    StartDebug,
    Stop,
    Start,
    Reset,
    Config(MouseConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DebugMsg {
    Hardware(HardwareDebug),
    Orientation(Orientation),
    MotionHandler(Option<MotionHandlerDebug>),
    MotionQueue(MotionQueueDebug),
    MotorControl(MotorControlDebug),
    Localize(LocalizeDebug),
    Slow(Option<SlowDebug>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugPacket {
    pub msgs: Vec<DebugMsg, U3>,
    pub battery: u16,
    pub time: u32,
    pub delta_time_sys: u32,
    pub delta_time_msg: u32,
    pub count: u16,
}
