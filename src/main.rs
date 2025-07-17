use eframe::egui::{self, Color32, ScrollArea};
use egui_plot::{Line, Plot, PlotBounds, PlotPoints,};

pub struct EllipticApp {
    pos_branch: Vec<[f64; 2]>,
    neg_branch: Vec<[f64; 2]>,
}

impl Default for EllipticApp {
    fn default() -> Self {
        Self {
            pos_branch: Vec::new(),
            neg_branch: Vec::new(),
        }
    }
}

impl EllipticApp {
    const NUM_POINTS: usize = 2000;
}

impl eframe::App for EllipticApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        fn pos_points<'a>() -> Line<'a> {
            Line::new(
                "positive points",
                PlotPoints::from_explicit_callback(
                    move  |x|  {
                        let y2 = x*x*x + 4.0;
                        y2.sqrt()
                    },
                    ..,
                    EllipticApp::NUM_POINTS

                )
            ).color(Color32::DARK_BLUE)

        }

        fn neg_points<'a>() -> Line<'a> {
            Line::new(
                "positive points",
                PlotPoints::from_explicit_callback(
                    move  |x|  {
                        let y2 = x*x*x + 4.0;
                        -y2.sqrt()
                    },
                    ..,
                    EllipticApp::NUM_POINTS

                )
            ).color(Color32::DARK_BLUE)
        }


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot of y² = x³ + 4");
            let plot = Plot::new("elliptic_plot")
                .legend(egui_plot::Legend::default())
                .show_axes(true)
                .show_grid(true);


            plot.show(ui, |plot_ui| {
                plot_ui.line(pos_points());
                plot_ui.line(neg_points());
            });
        });
    }
}

fn sample_curve(
    x_min: f64,
    x_max: f64,
    num_points: usize,
) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
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

    (pos_points, neg_points)
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Elliptic Curve Plot",
        native_options,
        Box::new(|_cc| Ok(Box::new(EllipticApp::default()))),
    )
}
