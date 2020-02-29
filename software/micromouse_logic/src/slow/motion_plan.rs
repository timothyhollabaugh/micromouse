use serde::{Deserialize, Serialize};

use heapless::Vec;

use crate::fast::motion_queue::{Motion, MotionQueueBuffer};
use crate::fast::path::PathMotion;
use crate::fast::turn::TurnMotion;

use crate::slow::maze::MazeConfig;
use crate::slow::{MazeDirection, MazeOrientation};

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionPlanConfig {}

pub fn motion_plan(
    _config: &MotionPlanConfig,
    maze_config: &MazeConfig,
    mouse: MazeOrientation,
    directions: &[MazeDirection],
) -> MotionQueueBuffer {
    let mut out = Vec::new();

    let mut current_orientation = mouse;

    for next_direction in directions.iter().copied() {
        let cell_center = current_orientation.position.center_position(maze_config);

        // Very dumb, but it should work.
        if next_direction.opposite() == current_orientation.direction {
            let half_cell = maze_config.cell_width / 2.0;
            let cell_start = match current_orientation.direction {
                MazeDirection::North => cell_center.offset_y(half_cell),
                MazeDirection::South => cell_center.offset_y(-half_cell),
                MazeDirection::East => cell_center.offset_x(half_cell),
                MazeDirection::West => cell_center.offset_x(-half_cell),
            };

            out.push(Motion::Path(PathMotion::line(cell_start, cell_center)))
                .ok();
            out.push(Motion::Turn(TurnMotion::new(
                current_orientation.direction.into_direction(),
                next_direction.into_direction(),
            )))
            .ok();
            out.push(Motion::Path(PathMotion::line(cell_center, cell_start)))
                .ok();
        } else {
            out.push(Motion::Path(PathMotion::corner(
                cell_center,
                current_orientation.direction.into_direction(),
                next_direction.into_direction(),
                90.0,
            )))
            .ok();
        };

        current_orientation.direction = next_direction;
        match next_direction {
            MazeDirection::North => current_orientation.position.y += 1,
            MazeDirection::South => current_orientation.position.y -= 1,
            MazeDirection::East => current_orientation.position.x += 1,
            MazeDirection::West => current_orientation.position.x -= 1,
        }
    }

    out.reverse();

    return out;
}
