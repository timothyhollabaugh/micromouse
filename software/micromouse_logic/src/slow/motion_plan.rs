use serde::{Deserialize, Serialize};

use heapless::Vec;

use crate::fast::motion_queue::{Motion, MotionQueueBuffer};
use crate::fast::path::PathMotion;
use crate::fast::turn::TurnMotion;

use crate::fast::Orientation;
use crate::slow::maze::MazeConfig;
use crate::slow::MazeDirection;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionPlanConfig {}

pub fn motion_plan(
    _config: &MotionPlanConfig,
    maze_config: &MazeConfig,
    orientation: Orientation,
    directions: &[MazeDirection],
) -> MotionQueueBuffer {
    let mut out = Vec::new();

    let mut current_orientation = orientation;

    for next_direction in directions.iter().copied() {
        let maze_orientation = current_orientation.to_maze_orientation(maze_config);
        let cell_center = maze_orientation.position.center_position(maze_config);

        let offset_distance = maze_config.cell_width / 2.0 + maze_config.wall_width / 2.0;
        let end_position = match next_direction {
            MazeDirection::North => cell_center.offset_y(offset_distance),
            MazeDirection::South => cell_center.offset_y(-offset_distance),
            MazeDirection::East => cell_center.offset_x(offset_distance),
            MazeDirection::West => cell_center.offset_x(-offset_distance),
        };

        // Very dumb, but it should work.

        // If we are not at the edge of the cell, stop and do a turn instead of a curve
        let center_threshold = maze_config.cell_width / 4.0;
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

        if do_manual_turn {
            out.push(Motion::Turn(TurnMotion::new(
                orientation.direction,
                next_direction.into_direction(),
            )))
            .ok();
            out.push(Motion::Path(PathMotion::line(cell_center, end_position)))
                .ok();
        } else {
            if next_direction.opposite() == maze_orientation.direction {
                let half_cell = maze_config.cell_width / 2.0;
                let cell_start = match maze_orientation.direction {
                    MazeDirection::North => cell_center.offset_y(-half_cell),
                    MazeDirection::South => cell_center.offset_y(half_cell),
                    MazeDirection::East => cell_center.offset_x(-half_cell),
                    MazeDirection::West => cell_center.offset_x(half_cell),
                };

                out.push(Motion::Path(PathMotion::line(cell_start, cell_center)))
                    .ok();
                out.push(Motion::Turn(TurnMotion::new(
                    maze_orientation.direction.into_direction(),
                    next_direction.into_direction(),
                )))
                .ok();
                out.push(Motion::Path(PathMotion::line(cell_center, cell_start)))
                    .ok();
            } else {
                out.push(Motion::Path(PathMotion::corner(
                    cell_center,
                    maze_orientation.direction.into_direction(),
                    next_direction.into_direction(),
                    maze_config.cell_width / 2.0,
                    maze_config.wall_width,
                )))
                .ok();
            }
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
    use crate::slow::MazeDirection;
    use heapless::Vec;

    #[test]
    fn u_turn() {
        let mut expected: MotionQueueBuffer = Vec::new();

        expected.push(Motion::Path(PathMotion::line(
            Vector { x: 180.0, y: 90.0 },
            Vector { x: 270.0, y: 90.0 },
        )));

        expected.push(Motion::Turn(TurnMotion::new(DIRECTION_0, DIRECTION_PI)));

        expected.push(Motion::Path(PathMotion::line(
            Vector { x: 270.0, y: 90.0 },
            Vector { x: 180.0, y: 90.0 },
        )));

        expected.reverse();

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
