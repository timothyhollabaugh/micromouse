use heapless::Vec;
use serde::Deserialize;
use serde::Serialize;
use typenum::consts::U2;

use crate::map::MapDebug;
use crate::map::Orientation;
use crate::motion::MotionDebug;
use crate::path::PathDebug;

#[derive(Debug, Serialize, Deserialize)]
pub enum DebugMsg {
    Orientation(Orientation),
    Path(PathDebug),
    Map(MapDebug),
    Motion(MotionDebug),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugPacket {
    pub msgs: Vec<DebugMsg, U2>,
    pub time: u32,
    pub count: u16,
}
