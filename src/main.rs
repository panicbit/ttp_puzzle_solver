use std::{fmt, iter};

use clap::Parser;
use fnv::{FnvHashMap, FnvHashSet};
use pastel::ansi::{Brush, Stream, Style};
use pastel::distinct::{distinct_colors, DistanceMetric};

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

    let square = Shape::from_str("##\n##", 0, '#');
    let line = Shape::from_str("####", 1, '+');
    let z = Shape::from_str("##\n ##", 1, 'Z');
    let reverse_z = Shape::from_str(" ##\n##", 1, 'N');
    let l = Shape::from_str("###\n#", 3, 'L');
    let reverse_l = Shape::from_str("#\n###", 3, '⅃');
    let t = Shape::from_str("###\n #", 3, 'T');

    let available_shapes = [
        (square, cli.square),
        (line, cli.line),
        (z, cli.z),
        (reverse_z, cli.reverse_z),
        (l, cli.l),
        (reverse_l, cli.reverse_l),
        (t, cli.t),
    ];
    let mut available_shapes = available_shapes
        .iter()
        .map(|(shape, amount)| (shape, *amount))
        .collect::<Vec<_>>();

    let width = i8::try_from(cli.width).expect("grid is too wide");
    let height = i8::try_from(cli.height).expect("grid is too tall");
    let mut grid = Grid::new(width, height);

    if !grid.fill_with_rec(&mut available_shapes, 0) {
        println!("failed to fill the grid! :(");
    }

    println!("{grid}");
}

struct Grid {
    width: i8,
    height: i8,
    cells: FnvHashMap<(i8, i8), (usize, char)>,
}

impl Grid {
    fn new(width: i8, height: i8) -> Self {
        Self {
            width,
            height,
            cells: <_>::default(),
        }
    }

    fn is_vacant(&self, (x, y): (i8, i8)) -> bool {
        if x < 0 || y < 0 {
            return false;
        }

        if x >= self.width || y >= self.height {
            return false;
        }

        !self.cells.contains_key(&(x, y))
    }

    fn fill_with_rec(&mut self, shapes: &mut [(&Shape, usize)], placement_index: usize) -> bool {
        if shapes.iter().all(|&(_, amount)| amount == 0) {
            return true;
        }

        for i in 0..shapes.len() {
            let amount = &mut shapes[i].1;

            if *amount == 0 {
                continue;
            }

            *amount -= 1;

            let shape = shapes[i].0;

            for pieces in shape.all_rotations() {
                let Some(placement_vector) = self.find_placement_vector(pieces) else {
                    continue;
                };

                self.place(pieces, placement_vector, placement_index, shape.glyph);

                // println!("{self}");

                if self.fill_with_rec(shapes, placement_index + 1) {
                    return true;
                }

                self.remove(pieces, placement_vector);
            }

            let amount = &mut shapes[i].1;

            *amount += 1;
        }

        false
    }

    fn find_placement_vector(&self, pieces: &FnvHashSet<(i8, i8)>) -> Option<(i8, i8)> {
        for grid_y in 0..self.height {
            for grid_x in 0..self.width {
                'next_origin: for (x_origin, y_origin) in pieces {
                    let placement_vector = (grid_x + x_origin, grid_y + y_origin);

                    for (x, y) in pieces {
                        let (x, y) = (placement_vector.0 - x, placement_vector.1 - y);

                        if !self.is_vacant((x, y)) {
                            continue 'next_origin;
                        }
                    }

                    return Some(placement_vector);
                }
            }
        }

        None
    }

    fn place(
        &mut self,
        pieces: &FnvHashSet<(i8, i8)>,
        placement_vector: (i8, i8),
        placement_index: usize,
        glyph: char,
    ) {
        for (x, y) in pieces {
            let x = placement_vector.0 - x;
            let y = placement_vector.1 - y;

            self.cells.insert((x, y), (placement_index, glyph));
        }
    }

    fn remove(&mut self, pieces: &FnvHashSet<(i8, i8)>, placement_vector: (i8, i8)) {
        for (x, y) in pieces {
            let x = placement_vector.0 - x;
            let y = placement_vector.1 - y;

            self.cells.remove(&(x, y));
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_placements = self
            .cells
            .values()
            .map(|(placement_index, _)| placement_index)
            .max()
            .copied()
            .unwrap_or(0)
            + 1;
        let num_placements = num_placements.max(2);
        let distance_metric = DistanceMetric::CIEDE2000;
        let fixed_colors = vec![];
        let brush = Brush::from_environment(Stream::Stdin);
        let (colors, _) =
            distinct_colors(num_placements, distance_metric, fixed_colors, &mut |_| {});

        write!(f, "┌")?;
        for _ in 0..self.width {
            write!(f, "─")?;
        }
        writeln!(f, "┐")?;

        for y in 0..self.height {
            write!(f, "│")?;

            for x in 0..self.width {
                let (placement_index, _glyph) =
                    self.cells.get(&(x, y)).copied().unwrap_or((0, ' '));
                let color = &colors[placement_index];
                let mut style = Style::default();
                style.on(color);

                write!(f, "{}", brush.paint(" ", style))?;
                // write!(f, "{glyph}")?;
            }

            writeln!(f, "│")?;
        }

        write!(f, "└")?;
        for _ in 0..self.width {
            write!(f, "─")?;
        }
        write!(f, "┘")?;

        Ok(())
    }
}

#[derive(Clone)]
struct Shape {
    pieces: FnvHashSet<(i8, i8)>,
    additional_rotations: Vec<FnvHashSet<(i8, i8)>>,
    glyph: char,
}

impl Shape {
    fn from_str(s: &str, mut num_additional_rotations: usize, glyph: char) -> Self {
        let mut pieces = FnvHashSet::default();

        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != '#' {
                    continue;
                }

                pieces.insert((x as i8, y as i8));
            }
        }

        let additional_rotations = iter::successors(Some(pieces.clone()), move |prev_pieces| {
            if num_additional_rotations == 0 {
                return None;
            };

            num_additional_rotations -= 1;

            Some(Self::rotate_pieces(prev_pieces))
        })
        .collect();

        Self {
            pieces,
            additional_rotations,
            glyph,
        }
    }

    fn rotate_pieces(pieces: &FnvHashSet<(i8, i8)>) -> FnvHashSet<(i8, i8)> {
        pieces.iter().map(|&(x, y)| (y, -x)).collect()
    }

    fn all_rotations(&self) -> impl Iterator<Item = &FnvHashSet<(i8, i8)>> {
        iter::once(&self.pieces).chain(&self.additional_rotations)
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min_x = self.pieces.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let min_y = self.pieces.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let max_x = self.pieces.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let max_y = self.pieces.iter().map(|(_, y)| *y).max().unwrap_or(0);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if self.pieces.contains(&(x, y)) {
                    write!(f, "{}", self.glyph)?;
                } else {
                    write!(f, " ")?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
