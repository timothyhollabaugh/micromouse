use libm::F32Ext;

use crate::config::MechanicalConfig;

pub struct MapConfig {
    pub cell_width: f32,
    pub wall_width: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn magnitude(&self) -> f32 {
        F32Ext::sqrt(self.x * self.x + self.y * self.y)
    }
}

impl core::ops::Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Orientation {
    pub position: Vector,
    pub direction: f32,
}

pub struct Map {
    orientation: Orientation,
    left_encoder: i32,
    right_encoder: i32,
}

impl Map {
    pub fn new(orientation: Orientation, left_encoder: i32, right_encoder: i32) -> Map {
        Map {
            orientation,
            left_encoder,
            right_encoder,
        }
    }

    pub fn update(
        &mut self,
        config: &MechanicalConfig,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Orientation {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        let delta_linear = config.ticks_to_mm((delta_right + delta_left) as f32 / 2.0);
        let delta_angular = config.ticks_to_rads((delta_right - delta_left) as f32 / 2.0);

        let mid_dir = self.orientation.direction + delta_angular / 2.0;

        let orientation = Orientation {
            position: Vector {
                x: self.orientation.position.x + delta_linear * F32Ext::cos(mid_dir),
                y: self.orientation.position.y + delta_linear * F32Ext::sin(mid_dir),
            },

            direction: self.orientation.direction + delta_angular,
        };

        self.orientation = orientation;

        self.orientation
    }
}
