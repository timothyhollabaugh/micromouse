use micromouse_logic::map::MapDebug;
use micromouse_logic::math::Direction;
use micromouse_logic::math::Orientation;
use micromouse_logic::math::Vector;
use micromouse_logic::maze::Maze;
use micromouse_logic::maze::Wall;
use micromouse_logic::maze::WallIndex;
use micromouse_logic::motion::MotionDebug;
use micromouse_logic::mouse::MouseDebug;
use micromouse_logic::path::PathBuf;
use micromouse_logic::path::PathDebug;
use micromouse_logic::path::Segment;

macro_rules! print_size {
    ($t:ty) => {
        println!("{}: {}", stringify!($t), std::mem::size_of::<$t>());
    };
}

fn main() {
    print_size!(Direction);
    print_size!(MapDebug);
    print_size!(Orientation);
    print_size!(Vector);
    print_size!(Wall);
    print_size!(WallIndex);
    print_size!(Maze);
    print_size!(MotionDebug);
    print_size!(MouseDebug);
    print_size!(PathBuf);
    print_size!(PathDebug);
    print_size!(Segment);
}
