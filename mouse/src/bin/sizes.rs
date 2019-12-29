use mouse::map::Direction;
use mouse::map::MapDebug;
use mouse::map::Orientation;
use mouse::map::Vector;
use mouse::maze::Edge;
use mouse::maze::EdgeIndex;
use mouse::maze::Maze;
use mouse::motion::MotionDebug;
use mouse::mouse::MouseDebug;
use mouse::path::PathBuf;
use mouse::path::PathDebug;
use mouse::path::Segment;

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
    print_size!(Edge);
    print_size!(EdgeIndex);
    print_size!(Maze);
    print_size!(MotionDebug);
    print_size!(MouseDebug);
    print_size!(PathBuf);
    print_size!(PathDebug);
    print_size!(Segment);
}
