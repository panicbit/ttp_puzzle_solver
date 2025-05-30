use clap::Parser;
use ttp_puzzle_solver::{Grid, ShapeSet};

#[derive(clap::Parser)]
struct Cli {
    width: u8,
    height: u8,
    #[clap(short, long, default_value_t = 0)]
    square: usize,
    #[clap(short = 'i', long, default_value_t = 0)]
    line: usize,
    #[clap(short, long, default_value_t = 0)]
    z: usize,
    #[clap(long = "rz", default_value_t = 0)]
    reverse_z: usize,
    #[clap(short, long, default_value_t = 0)]
    l: usize,
    #[clap(long = "rl", default_value_t = 0)]
    reverse_l: usize,
    #[clap(short, long, default_value_t = 0)]
    t: usize,
}

fn main() {
    let cli = Cli::parse();
    let shapeset = ShapeSet {
        square: cli.square,
        line: cli.line,
        z: cli.z,
        reverse_z: cli.reverse_z,
        l: cli.l,
        reverse_l: cli.reverse_l,
        t: cli.t,
    };
    let mut available_shapes = shapeset.to_shapes();

    let width = i8::try_from(cli.width).expect("grid is too wide");
    let height = i8::try_from(cli.height).expect("grid is too tall");
    let mut grid = Grid::new(width, height);

    if !grid.fill_with_rec(&mut available_shapes, 0) {
        println!("failed to fill the grid! :(");
    }

    println!("{grid}");
}
