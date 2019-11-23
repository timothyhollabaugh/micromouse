use core::f32::consts::PI;
use core::fmt::{Error, Formatter};
use core::ops::Mul;

use libm::F32Ext;

use crate::config::MechanicalConfig;
use crate::maze::Edge;
use crate::maze::EdgeIndex;
use crate::maze::Maze;
use crate::maze::MazeConfig;

#[derive(Debug, Copy, Clone)]
pub struct MapConfig {
    pub maze: MazeConfig,
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

    pub fn direction(&self) -> Direction {
        Direction::from(F32Ext::atan2(self.y, self.x))
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

impl core::ops::Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl core::ops::Mul<Vector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl core::ops::Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl core::ops::AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

/// A direction wrapped to 0 - 2pi
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Direction(f32);

impl Direction {
    pub fn centered_at(self, center: Direction) -> f32 {
        let raw_self = f32::from(self);
        let raw_center = f32::from(center);

        let diff = raw_self - raw_center;

        if diff > PI {
            raw_self - 2.0 * PI
        } else if diff < -PI {
            raw_self + 2.0 * PI
        } else {
            raw_self
        }
    }

    pub fn into_unit_vector(self) -> Vector {
        Vector {
            x: F32Ext::cos(self.0),
            y: F32Ext::sin(self.0),
        }
    }
}

impl From<f32> for Direction {
    fn from(other: f32) -> Direction {
        Direction((other + 2.0 * PI) % (2.0 * PI))
    }
}

impl core::fmt::Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<Direction> for f32 {
    fn from(other: Direction) -> f32 {
        other.0
    }
}

impl From<Direction> for f64 {
    fn from(other: Direction) -> f64 {
        other.0 as f64
    }
}

impl core::ops::AddAssign for Direction {
    fn add_assign(&mut self, rhs: Self) {
        *self = Direction::from(self.0 + rhs.0)
    }
}

impl core::ops::Add for Direction {
    type Output = Direction;

    fn add(self, rhs: Self) -> Self::Output {
        Direction::from(self.0 + rhs.0)
    }
}

impl core::ops::Sub for Direction {
    type Output = Direction;

    fn sub(self, rhs: Self) -> Self::Output {
        Direction::from(self.0 - rhs.0)
    }
}

impl core::ops::Div<f32> for Direction {
    type Output = Direction;

    fn div(self, rhs: f32) -> Self::Output {
        Direction::from(self.0 / rhs)
    }
}

pub const DIRECTION_PI_2: Direction = Direction(core::f32::consts::FRAC_PI_2);
pub const DIRECTION_PI: Direction = Direction(core::f32::consts::PI);

#[derive(Copy, Clone, Debug)]
pub struct Orientation {
    pub position: Vector,
    pub direction: Direction,
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

        let mid_dir = f32::from(self.direction) + delta_angular / 2.0;

        self.position.x += delta_linear * F32Ext::cos(mid_dir);
        self.position.y += delta_linear * F32Ext::sin(mid_dir);
        self.direction = self.direction + Direction::from(delta_angular);
    }
}

#[derive(Clone, Debug)]
pub struct MapDebug {
    pub maze: Maze,
    pub front_edge: Option<EdgeIndex>,
}

pub struct Map {
    orientation: Orientation,
    maze: Maze,
    left_encoder: i32,
    right_encoder: i32,
}

impl Map {
    pub fn new(orientation: Orientation, left_encoder: i32, right_encoder: i32) -> Map {
        let mut horizontal_edges = [[Edge::Unknown; crate::maze::HEIGHT - 1]; crate::maze::WIDTH];
        let mut vertical_edges = [[Edge::Unknown; crate::maze::HEIGHT]; crate::maze::WIDTH - 1];

        horizontal_edges[6][6] = Edge::Closed;
        horizontal_edges[7][6] = Edge::Closed;
        horizontal_edges[8][6] = Edge::Closed;
        horizontal_edges[9][6] = Edge::Closed;

        horizontal_edges[6][7] = Edge::Open;
        horizontal_edges[7][7] = Edge::Closed;
        horizontal_edges[8][7] = Edge::Closed;
        horizontal_edges[9][7] = Edge::Open;

        horizontal_edges[6][8] = Edge::Closed;
        horizontal_edges[7][8] = Edge::Closed;
        horizontal_edges[8][8] = Edge::Closed;
        horizontal_edges[9][8] = Edge::Closed;

        vertical_edges[5][7] = Edge::Closed;
        vertical_edges[5][8] = Edge::Closed;

        vertical_edges[6][7] = Edge::Open;
        vertical_edges[6][8] = Edge::Open;

        vertical_edges[7][7] = Edge::Open;
        vertical_edges[7][8] = Edge::Open;

        vertical_edges[8][7] = Edge::Open;
        vertical_edges[8][8] = Edge::Open;

        vertical_edges[9][7] = Edge::Closed;
        vertical_edges[9][8] = Edge::Closed;

        let maze = Maze::from_edges(horizontal_edges, vertical_edges);

        Map {
            orientation,
            left_encoder,
            right_encoder,
            maze,
        }
    }

    pub fn update(
        &mut self,
        mech_config: &MechanicalConfig,
        maze_config: &MazeConfig,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
    ) -> (Orientation, MapDebug) {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        self.orientation
            .update_from_encoders(&mech_config, delta_left, delta_right);

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;

        let front_edge = maze_config
            .edge_projection_iter(self.orientation)
            .find(|edge_index| {
                *self.maze.get_edge(*edge_index).unwrap_or(&Edge::Closed) == Edge::Closed
            });

        let debug = MapDebug {
            maze: self.maze.clone(),
            front_edge: front_edge,
        };

        (self.orientation, debug)
    }
}
