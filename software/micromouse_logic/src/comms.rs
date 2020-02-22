use heapless::Vec;
use serde::Deserialize;
use serde::Serialize;
use typenum::consts::U2;

use crate::map::MapDebug;
use crate::math::Orientation;
use crate::motion::MotionDebug;
use crate::mouse::{HardwareDebug, MouseConfig};
use crate::path::PathDebug;

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
    Path(PathDebug),
    Map(MapDebug),
    Motion(MotionDebug),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugPacket {
    pub msgs: Vec<DebugMsg, U2>,
    pub battery: u16,
    pub time: u32,
    pub delta_time_sys: u32,
    pub delta_time_msg: u32,
    pub count: u16,
}
