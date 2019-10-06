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

impl Orientation {
    pub fn update_from_encoders(
        &mut self,
        config: &MechanicalConfig,
        delta_left: i32,
        delta_right: i32,
    ) {
        let delta_linear = config.ticks_to_mm((delta_right + delta_left) as f32 / 2.0);
        let delta_angular = config.ticks_to_rads((delta_right - delta_left) as f32 / 2.0);

        let mid_dir = self.direction + delta_angular / 2.0;

        self.position.x += delta_linear * F32Ext::cos(mid_dir);
        self.position.y += delta_linear * F32Ext::sin(mid_dir);
        self.direction += delta_angular;
    }
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

        self.orientation
            .update_from_encoders(&config, delta_left, delta_right);

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;

        self.orientation
    }
}
