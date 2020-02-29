use postcard;

use serde::Deserialize;
use serde::Serialize;

use micromouse_logic::comms::DebugMsg;
use micromouse_logic::comms::DebugPacket;

use micromouse_logic::mouse::MouseConfig;
use micromouse_logic::mouse::MouseDebug;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub mouse: MouseConfig,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RemoteDebug {
    mouse: MouseDebug,
    delta_time_msg: u32,
    config: RemoteConfig,
    bytes: usize,
}

pub struct Remote {
    debug: RemoteDebug,
    buf: Vec<u8>,
}

impl Remote {
    pub fn new(config: &RemoteConfig) -> Remote {
        let debug = RemoteDebug {
            config: config.clone(),
            ..RemoteDebug::default()
        };
        Remote {
            debug,
            buf: Vec::new(),
        }
    }

    pub fn default_config() -> RemoteConfig {
        RemoteConfig::default()
    }

    pub fn update(&mut self, bytes: &[u8]) -> Result<Vec<RemoteDebug>, ()> {
        let mut debugs = Vec::new();

        for &byte in bytes {
            self.buf.push(byte);
            match postcard::take_from_bytes::<DebugPacket>(&self.buf) {
                Ok((packet, remaining)) => {
                    self.debug.bytes = self.buf.len() - remaining.len();
                    self.buf = Vec::from(remaining.clone());

                    self.debug.mouse.time = packet.time;
                    self.debug.mouse.delta_time = packet.delta_time_sys;
                    self.debug.delta_time_msg = packet.delta_time_msg;
                    self.debug.mouse.battery = packet.battery;

                    for msg in packet.msgs {
                        match msg {
                            DebugMsg::Hardware(hardware) => {
                                self.debug.mouse.hardware = hardware
                            }
                            DebugMsg::Orientation(orientation) => {
                                self.debug.mouse.orientation = orientation
                            }
                            DebugMsg::Slow(slow) => self.debug.mouse.slow = slow,
                            DebugMsg::MotorControl(motor_control) => {
                                self.debug.mouse.motion_control.motor_control =
                                    motor_control
                            }
                            DebugMsg::MotionHandler(handler) => {
                                self.debug.mouse.motion_control.handler = handler
                            }
                            DebugMsg::MotionQueue(queue) => {
                                self.debug.mouse.motion_queue = queue
                            }
                        }
                    }

                    debugs.push(self.debug.clone());
                }
                Err(postcard::Error::DeserializeUnexpectedEnd) => {}
                Err(_e) => {
                    self.buf = Vec::new();
                    return Err(());
                }
            }
        }

        Ok(debugs)
    }
}
