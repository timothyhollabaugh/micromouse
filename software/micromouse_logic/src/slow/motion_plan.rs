use serde::{Deserialize, Serialize};

use heapless::Vec;

use crate::fast::motion_queue::{Motion, MotionQueueBuffer};
use crate::fast::path::PathMotion;
use crate::fast::turn::TurnMotion;

use crate::fast::Orientation;
use crate::slow::maze::MazeConfig;
use crate::slow::MazeDirection;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionPlanConfig {
    /// How much to offset the start of a move into the current cell and the end of a move into the
    /// next cell
    pub move_offset: f32,
}

pub fn motion_plan(
    config: &MotionPlanConfig,
    maze_config: &MazeConfig,
    orientation: Orientation,
    directions: &[MazeDirection],
) -> MotionQueueBuffer {
    let mut out = Vec::new();

    let mut current_orientation = orientation;

    for next_direction in directions.iter().copied() {
        let maze_orientation = current_orientation.to_maze_orientation(maze_config);
        let cell_center = maze_orientation.position.center_position(maze_config);

        let offset_distance = maze_config.cell_width / 2.0 + config.move_offset;
        let end_position = match next_direction {
            MazeDirection::North => cell_center.offset_y(offset_distance),
            MazeDirection::South => cell_center.offset_y(-offset_distance),
            MazeDirection::East => cell_center.offset_x(offset_distance),
            MazeDirection::West => cell_center.offset_x(-offset_distance),
        };

        // Very dumb, but it should work.

        // If we are not at the edge of the cell, stop and do a turn instead of a curve
        let center_threshold = maze_config.cell_width / 8.0;
        let do_manual_turn = match maze_orientation.direction {
            MazeDirection::North => {
                current_orientation.position.y > cell_center.y - center_threshold
            }
            MazeDirection::South => {
                current_orientation.position.y < cell_center.y + center_threshold
            }
            MazeDirection::East => {
                current_orientation.position.x > cell_center.x - center_threshold
            }
            MazeDirection::West => {
                current_orientation.position.x < cell_center.x + center_threshold
            }
        };

        if do_manual_turn || next_direction.opposite() == maze_orientation.direction {
            out.push(Motion::Turn(TurnMotion::new(
                orientation.direction,
                next_direction.into_direction(),
            )))
            .ok();
            out.push(Motion::Path(PathMotion::line(cell_center, end_position)))
                .ok();
        } else {
            out.push(Motion::Path(PathMotion::corner(
                cell_center,
                maze_orientation.direction.into_direction(),
                next_direction.into_direction(),
                maze_config.cell_width / 2.0,
                config.move_offset,
            )))
            .ok();
        }

        current_orientation.direction = next_direction.into_direction();

        current_orientation.position = end_position;
    }

    out.reverse();

    return out;
}

#[cfg(test)]
mod test_motion_plan {
    #[allow(unused_imports)]
    use crate::test::*;

    use pretty_assertions::assert_eq;

    use super::motion_plan;
    use crate::config::MAZE;
    use crate::config::MOTION_PLAN;
    use crate::fast::motion_queue::{Motion, MotionQueueBuffer};
    use crate::fast::path::PathMotion;
    use crate::fast::turn::TurnMotion;
    use crate::fast::{Orientation, Vector, DIRECTION_0, DIRECTION_PI};
    use crate::slow::motion_plan::MotionPlanConfig;
    use crate::slow::MazeDirection;
    use heapless::Vec;

    const CONFIG: MotionPlanConfig = MotionPlanConfig { move_offset: 12.0 };

    #[test]
    fn u_turn() {
        let mut expected: MotionQueueBuffer = Vec::new();
        expected
            .push(Motion::Path(PathMotion::line(
                Vector {
                    x: 180.0 + 90.0,
                    y: 90.0,
                },
                Vector {
                    x: 180.0 - CONFIG.move_offset,
                    y: 90.0,
                },
            )))
            .ok();
        expected
            .push(Motion::Turn(TurnMotion::new(DIRECTION_0, DIRECTION_PI)))
            .ok();

        assert_eq!(
            motion_plan(
                &MOTION_PLAN,
                &MAZE,
                Orientation {
                    position: Vector { x: 180.0, y: 90.0 },
                    direction: DIRECTION_0,
                },
                &[MazeDirection::West]
            ),
            expected
        )
    }
}
