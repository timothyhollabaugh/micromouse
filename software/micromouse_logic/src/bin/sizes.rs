use micromouse_logic::fast::{Direction, Orientation, Vector};

use micromouse_logic::comms::DebugMsg;
use micromouse_logic::comms::DebugPacket;
use micromouse_logic::fast::motion_control::MotionControlDebug;
use micromouse_logic::fast::motion_queue::Motion;
use micromouse_logic::fast::motion_queue::MotionQueueBuffer;
use micromouse_logic::fast::motion_queue::MotionQueueDebug;
use micromouse_logic::fast::path::PathHandlerDebug;
use micromouse_logic::fast::turn::TurnHandlerDebug;
use micromouse_logic::slow::navigate::TwelvePartitionNavigateDebug;
use micromouse_logic::slow::MazeDirection;
use micromouse_logic::slow::MazeOrientation;
use micromouse_logic::slow::MazePosition;

macro_rules! print_size {
    ($t:ty) => {
        println!("{}: {}", stringify!($t), std::mem::size_of::<$t>());
    };
}

fn main() {
    print_size!(Orientation);
    print_size!(Vector);
    print_size!(Direction);
    print_size!(PathHandlerDebug);
    print_size!(TurnHandlerDebug);
    print_size!(MotionQueueDebug);
    print_size!(MotionQueueBuffer);
    print_size!(MotionControlDebug);
    print_size!(Motion);
    print_size!(MazeOrientation);
    print_size!(MazeDirection);
    print_size!(MazePosition);
    print_size!(TwelvePartitionNavigateDebug);
    print_size!(DebugMsg);
    print_size!(DebugPacket);
}
