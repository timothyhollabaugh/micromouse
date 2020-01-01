use postcard;

use serde::Deserialize;
use serde::Serialize;

use mouse::comms::DebugMsg;
use mouse::comms::DebugPacket;

use mouse::mouse::MouseConfig;
use mouse::mouse::MouseDebug;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RemoteConfig {
    mouse: MouseConfig,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RemoteDebug {
    mouse: MouseDebug,
    config: RemoteConfig,
}

pub struct Remote {
    config: RemoteConfig,
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
            config: config.clone(),
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
                    self.buf = Vec::from(remaining.clone());

                    self.debug.mouse.time = packet.time;

                    for msg in packet.msgs {
                        match msg {
                            DebugMsg::Orientation(orientation) => {
                                self.debug.mouse.orientation = orientation
                            }
                            DebugMsg::Map(map) => self.debug.mouse.map = map,
                            DebugMsg::Motion(motion) => self.debug.mouse.motion = motion,
                            DebugMsg::Path(path) => self.debug.mouse.path = path,
                        }
                    }

                    debugs.push(self.debug.clone());
                }
                Err(postcard::Error::DeserializeUnexpectedEnd) => {}
                Err(e) => {
                    self.buf = Vec::new();
                    return Err(());
                }
            }
        }

        Ok(debugs)
    }
}
