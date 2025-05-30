use std::{fmt, iter};

use fnv::FnvHashSet;
use pastel::ansi::{Brush, Stream, Style};
use pastel::distinct::{distinct_colors, DistanceMetric};

pub struct Grid {
    cells: Array2<Option<(usize, char)>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: Array2::new(width, height),
        }
    }

    pub fn width(&self) -> usize {
        self.cells.width
    }

    pub fn height(&self) -> usize {
        self.cells.height
    }

    fn is_vacant(&self, (x, y): (isize, isize)) -> bool {
        let Some(cell) = self.cells.get((x, y)) else {
            return false;
        };

        cell.is_none()
    }

    pub fn fill_with_rec(
        &mut self,
        shapes: &mut [(&Shape, usize)],
        placement_index: usize,
    ) -> bool {
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

    fn find_placement_vector(&self, pieces: &FnvHashSet<(isize, isize)>) -> Option<(isize, isize)> {
        for grid_y in 0..self.height() as isize {
            for grid_x in 0..self.width() as isize {
                'next_origin: for (x_origin, y_origin) in pieces {
                    let placement_vector = (grid_x + x_origin, grid_y + y_origin);

                    for (x, y) in pieces {
                        let (x, y) = (placement_vector.0 - x, placement_vector.1 - y);

                        if x < 0
                            || y < 0
                            || x >= self.width() as isize
                            || y >= self.height() as isize
                        {
                            assert!(!self.is_vacant((x, y)), "vacant for {x}, {y}");
                        }

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
        pieces: &FnvHashSet<(isize, isize)>,
        placement_vector: (isize, isize),
        placement_index: usize,
        glyph: char,
    ) {
        for (x, y) in pieces {
            let x = placement_vector.0 - x;
            let y = placement_vector.1 - y;

            self.cells.set((x, y), Some((placement_index, glyph)));
        }
    }

    fn remove(&mut self, pieces: &FnvHashSet<(isize, isize)>, placement_vector: (isize, isize)) {
        for (x, y) in pieces {
            let x = placement_vector.0 - x;
            let y = placement_vector.1 - y;

            self.cells.set((x, y), None);
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_placements = self
            .cells
            .values()
            .flatten()
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
        for _ in 0..self.width() {
            write!(f, "─")?;
        }
        writeln!(f, "┐")?;

        for y in 0..self.height() {
            write!(f, "│")?;

            for x in 0..self.width() {
                let (placement_index, _glyph) = self
                    .cells
                    .get((x, y))
                    .copied()
                    .flatten()
                    .unwrap_or((0, ' '));
                let color = &colors[placement_index];
                let mut style = Style::default();
                style.on(color);

                write!(f, "{}", brush.paint(" ", style))?;
                // write!(f, "{glyph}")?;
            }

            writeln!(f, "│")?;
        }

        write!(f, "└")?;
        for _ in 0..self.width() {
            write!(f, "─")?;
        }
        write!(f, "┘")?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Shape {
    pieces: FnvHashSet<(isize, isize)>,
    additional_rotations: Vec<FnvHashSet<(isize, isize)>>,
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

                pieces.insert((x as isize, y as isize));
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

    fn rotate_pieces(pieces: &FnvHashSet<(isize, isize)>) -> FnvHashSet<(isize, isize)> {
        pieces.iter().map(|&(x, y)| (y, -x)).collect()
    }

    fn all_rotations(&self) -> impl Iterator<Item = &FnvHashSet<(isize, isize)>> {
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

pub struct ShapeSet {
    pub square: usize,
    pub line: usize,
    pub z: usize,
    pub reverse_z: usize,
    pub l: usize,
    pub reverse_l: usize,
    pub t: usize,
}

impl ShapeSet {
    pub fn to_shapes(&self) -> [(&'static Shape, usize); 7] {
        [
            (&shape::SQUARE, self.square),
            (&shape::LINE, self.line),
            (&shape::Z, self.z),
            (&shape::REVERSE_Z, self.reverse_z),
            (&shape::L, self.l),
            (&shape::REVERSE_L, self.reverse_l),
            (&shape::T, self.t),
        ]
    }
}

pub mod shape {
    use std::sync::LazyLock;

    use crate::Shape;

    pub static SQUARE: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("##\n##", 0, '#'));
    pub static LINE: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("####", 1, '+'));
    pub static Z: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("##\n ##", 1, 'Z'));
    pub static REVERSE_Z: LazyLock<Shape> = LazyLock::new(|| Shape::from_str(" ##\n##", 1, 'N'));
    pub static L: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("###\n#", 3, 'L'));
    pub static REVERSE_L: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("#\n###", 3, '⅃'));
    pub static T: LazyLock<Shape> = LazyLock::new(|| Shape::from_str("###\n #", 3, 'T'));
}

pub struct Array2<T> {
    inner: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Array2<T> {
    pub fn new(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        let size = width * height;
        let mut inner = Vec::with_capacity(size);

        for _ in 0..size {
            inner.push(T::default());
        }

        debug_assert_eq!(inner.len(), size);

        Self {
            inner,
            width,
            height,
        }
    }

    fn linear_index(&self, index: impl Array2Index) -> Option<usize> {
        index.linear_index(self.width, self.height)
    }

    pub fn get(&self, index: impl Array2Index) -> Option<&T> {
        self.inner.get(self.linear_index(index)?)
    }

    pub fn set(&mut self, index: impl Array2Index, element: T) {
        let Some(index) = self.linear_index(index) else {
            return;
        };

        let Some(cell) = self.inner.get_mut(index) else {
            return;
        };

        *cell = element;
    }

    fn values(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub trait Array2Index {
    fn linear_index(self, width: usize, height: usize) -> Option<usize>;
}

impl Array2Index for (usize, usize) {
    fn linear_index(self, width: usize, height: usize) -> Option<usize> {
        let (x, y) = self;

        if x >= width || y >= height {
            return None;
        }

        width.checked_mul(y)?.checked_add(x)
    }
}

impl Array2Index for (i32, i32) {
    fn linear_index(self, width: usize, height: usize) -> Option<usize> {
        let (x, y) = self;
        let x = usize::try_from(x).ok()?;
        let y = usize::try_from(y).ok()?;

        (x, y).linear_index(width, height)
    }
}

impl Array2Index for (isize, isize) {
    fn linear_index(self, width: usize, height: usize) -> Option<usize> {
        let (x, y) = self;
        let x = usize::try_from(x).ok()?;
        let y = usize::try_from(y).ok()?;

        (x, y).linear_index(width, height)
    }
}
