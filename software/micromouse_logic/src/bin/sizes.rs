macro_rules! print_size {
    ($t:ty) => {
        println!("{}: {}", stringify!($t), std::mem::size_of::<$t>());
    };
}

fn main() {}
