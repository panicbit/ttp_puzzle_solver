use eframe::egui::{self, Color32, Grid, Sense, Stroke, StrokeKind, Ui, Vec2, vec2};
use pastel::distinct::{DistanceMetric, distinct_colors};
use ttp_puzzle_solver::ShapeSet;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Milton's Sigil Solver",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap()
}

struct App {
    sigil_set: ShapeSet,
    grid: ttp_puzzle_solver::Grid,
    grid_colors: Vec<Color32>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.show_main(ui));
    }
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            sigil_set: ShapeSet {
                square: 1,
                line: 2,
                z: 2,
                reverse_z: 1,
                l: 2,
                reverse_l: 0,
                t: 2,
            },
            grid: ttp_puzzle_solver::Grid::new(8, 5),
            grid_colors: vec![],
        }
    }

    fn get_sigil_count(&self, sigil: Sigil) -> usize {
        match sigil {
            Sigil::Square => self.sigil_set.square,
            Sigil::Line => self.sigil_set.line,
            Sigil::Z => self.sigil_set.z,
            Sigil::ReverseZ => self.sigil_set.reverse_z,
            Sigil::L => self.sigil_set.l,
            Sigil::ReverseL => self.sigil_set.reverse_l,
            Sigil::T => self.sigil_set.t,
        }
    }

    fn get_sigil_count_mut(&mut self, sigil: Sigil) -> &mut usize {
        match sigil {
            Sigil::Square => &mut self.sigil_set.square,
            Sigil::Line => &mut self.sigil_set.line,
            Sigil::Z => &mut self.sigil_set.z,
            Sigil::ReverseZ => &mut self.sigil_set.reverse_z,
            Sigil::L => &mut self.sigil_set.l,
            Sigil::ReverseL => &mut self.sigil_set.reverse_l,
            Sigil::T => &mut self.sigil_set.t,
        }
    }

    fn fill(&mut self) {
        let available_shapes = self.sigil_set.to_shapes();

        self.grid.fill_with(&available_shapes);

        let num_colors = self.grid.num_placements().max(2);
        let distance_metric = DistanceMetric::CIEDE2000;
        let fixed_colors = vec![];
        let (colors, _) = distinct_colors(num_colors, distance_metric, fixed_colors, &mut |_| {});
        self.grid_colors = colors
            .into_iter()
            .map(|color| {
                let pastel::RGBA { r, g, b, alpha: _ } = color.to_rgba();

                Color32::from_rgb(r, g, b)
            })
            .collect::<Vec<_>>();
    }

    fn set_grid_width(&mut self, width: usize) {
        self.grid = ttp_puzzle_solver::Grid::new(width, self.grid.height());
    }

    fn set_grid_height(&mut self, height: usize) {
        self.grid = ttp_puzzle_solver::Grid::new(self.grid.width(), height);
    }
}

impl App {
    fn show_main(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.group(|ui| {
                    Grid::new("grid_size")
                        .min_col_width(5.)
                        .striped(true)
                        .show(ui, |ui| {
                            if let Some(new_width) = counter(ui, "Width", self.grid.width()) {
                                self.set_grid_width(new_width);
                            }
                            ui.end_row();
                            if let Some(new_height) = counter(ui, "Height", self.grid.height()) {
                                self.set_grid_height(new_height);
                            }
                            ui.end_row();
                        });
                });

                ui.group(|ui| {
                    Grid::new("sigil_counters")
                        .min_col_width(5.)
                        .striped(true)
                        .show(ui, |ui| {
                            self.sigil_counter(ui, Sigil::Square);
                            self.sigil_counter(ui, Sigil::Line);
                            self.sigil_counter(ui, Sigil::Z);
                            self.sigil_counter(ui, Sigil::ReverseZ);
                            self.sigil_counter(ui, Sigil::L);
                            self.sigil_counter(ui, Sigil::ReverseL);
                            self.sigil_counter(ui, Sigil::T);
                        });
                });

                if ui.button("Fill!").clicked() {
                    self.fill();
                }
            });

            let piece_size = Vec2::splat(25.);
            let grid_size = piece_size * vec2(self.grid.width() as f32, self.grid.height() as f32);

            let (response, painter) = ui.allocate_painter(grid_size, Sense::empty());
            let rect = response.rect;
            let corner_radius = 0;
            let stroke_width = 1.;
            let stroke_color = Color32::from_rgb(15, 15, 15);
            let stroke = Stroke::new(stroke_width, stroke_color);
            let stroke_kind = StrokeKind::Inside;

            for y in 0..self.grid.height() {
                for x in 0..self.grid.width() {
                    let placement_index = self.grid.get_placement_index(x, y);
                    let fill_color = self
                        .grid_colors
                        .get(placement_index)
                        .copied()
                        .unwrap_or(Color32::from_rgb(50, 50, 50));
                    let stroke = Stroke::new(stroke_width, stroke_color);
                    let stroke_kind = StrokeKind::Outside;

                    let mut rect = rect;
                    rect.set_width(piece_size.x);
                    rect.set_height(piece_size.y);
                    let rect = rect.translate(piece_size * vec2(x as f32, y as f32));

                    painter.rect(rect, corner_radius, fill_color, stroke, stroke_kind);
                }
            }

            let fill_color = Color32::TRANSPARENT;
            painter.rect(rect, corner_radius, fill_color, stroke, stroke_kind);
        });
    }

    fn sigil_counter(&mut self, ui: &mut Ui, sigil: Sigil) {
        let label = sigil_label(sigil);
        let count = self.get_sigil_count_mut(sigil);

        if let Some(new_count) = counter(ui, label, *count) {
            *count = new_count;
        }

        ui.end_row();
    }
}

fn sigil_label(sigil: Sigil) -> &'static str {
    match sigil {
        Sigil::Square => "S",
        Sigil::Line => "I",
        Sigil::Z => "Z",
        Sigil::ReverseZ => "RZ",
        Sigil::L => "L",
        Sigil::ReverseL => "RL",
        Sigil::T => "T",
    }
}

fn counter(ui: &mut Ui, label: &str, value: usize) -> Option<usize> {
    ui.label(label);
    let mut changed_value = None;

    if ui.button("-").clicked() {
        changed_value = Some(value.saturating_sub(1));
    }

    ui.label(value.to_string());

    if ui.button("+").clicked() {
        changed_value = Some(value.saturating_add(1));
    }

    changed_value
}

#[derive(Debug, Copy, Clone)]
pub enum Sigil {
    Square,
    Line,
    Z,
    ReverseZ,
    L,
    ReverseL,
    T,
}

struct FillResult {
    grid: ttp_puzzle_solver::Grid,
    colors: Vec<Color32>,
}
