use eframe::egui::{self, Color32};
use egui_plot::{Line, Plot, PlotBounds, PlotPoints};

pub struct EllipticApp;

impl eframe::App for EllipticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot of y² = x³ + 4");

            // Capture the pointer position
            let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

            // Check whether the mouse is inside this panel
            let in_bounds = pointer_pos
                .map(|pos| ui.min_rect().contains(pos))
                .unwrap_or(false);

            // if in_bounds {

            // }


            Plot::new("elliptic_plot")
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    let bounds: PlotBounds = plot_ui.plot_bounds();
                    let x_min = bounds.min()[0];
                    let x_max = bounds.max()[0];

                    let (pos_branch, neg_branch) = sample_curve(x_min, x_max, 1000);

                    let line_pos = Line::new("y = √(x³ + 4)", pos_branch)
                        .color(Color32::DARK_BLUE);
                    let line_neg = Line::new("y = -√(x³ + 4)", neg_branch)
                        .color(Color32::DARK_BLUE);

                    plot_ui.line(line_pos);
                    plot_ui.line(line_neg);
                });
        });
    }
}

fn sample_curve<'a>(x_min: f64, x_max: f64, num_points: usize) -> (PlotPoints<'a>, PlotPoints<'a>) {
    let step = (x_max - x_min) / num_points as f64;

    let mut pos_points = Vec::with_capacity(num_points);
    let mut neg_points = Vec::with_capacity(num_points);

    for i in 0..=num_points {
        let x = x_min + i as f64 * step;
        let rhs = x * x * x + 4.0;
        if rhs >= 0.0 {
            let y = rhs.sqrt();
            pos_points.push([x, y]);
            neg_points.push([x, -y]);
        }
    }

    (PlotPoints::from(pos_points), PlotPoints::from(neg_points))
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Elliptic Curve Plot (Hover-sensitive)",
        options,
        Box::new(|_cc| Ok(Box::new(EllipticApp))),
    )
}
