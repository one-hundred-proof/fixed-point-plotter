use eframe::egui::{self};
use egui_plot::{Plot, PlotPoints, Points};
use primitive_types::{U256};

mod maths;
mod functions;

use crate::functions::{yearn_calc_supply};
use crate::maths::*;

const NUM_POINTS: usize = 5000;

const X_RADIX: u8   = 10;
const X_PLACES: u32 = 18;
const X_MIN: f64    = 0.0;
const X_MAX: f64    = 1.0;

const Y_RADIX: u8   = 10;
const Y_PLACES: u32 = 18;
const Y_MIN: f64    = 0.0;
const Y_MAX: f64    = 1.0;

const plot_fun: fn(U256) -> U256 = yearn_calc_supply;

pub struct EllipticApp {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    x_min_input: String,
    x_max_input: String,
    y_min_input: String,
    y_max_input: String,
    current_bounds: Option<egui_plot::PlotBounds>,
    reset_view: bool,
}

impl Default for EllipticApp {
    fn default() -> Self {
        /* These bounds must be pre-divided by radix^places */
        let (x_min, x_max, y_min, y_max) = (X_MIN, X_MAX, Y_MIN, Y_MAX);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            x_min_input: x_min.to_string(),
            x_max_input: x_max.to_string(),
            y_min_input: y_min.to_string(),
            y_max_input: y_max.to_string(),
            current_bounds: None,
            reset_view: false,
        }
    }
}

impl eframe::App for EllipticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("input_panel").show(ctx, |ui| {
            ui.heading("Plot (U256 scaled)");

            // Add controls for adjusting bounds
            ui.horizontal(|ui| {
                ui.label("X Min:");
                ui.text_edit_singleline(&mut self.x_min_input);
                if let Ok(value) = self.x_min_input.parse::<f64>() {
                    if self.x_min != value {
                        self.reset_view = true;
                        self.x_min = value;
                    }
                    ui.label(format!("({:.2e})", self.x_min));
                }

                ui.label("X Max:");
                ui.text_edit_singleline(&mut self.x_max_input);
                if let Ok(value) = self.x_max_input.parse::<f64>() {
                    if self.x_max != value {
                        self.reset_view = true;
                        self.x_max = value;
                    }
                    ui.label(format!("({:.2e})", self.x_max));
                }
            });

            ui.horizontal(|ui| {
                ui.label("Y Min:");
                ui.text_edit_singleline(&mut self.y_min_input);
                if let Ok(value) = self.y_min_input.parse::<f64>() {
                    if self.y_min != value {
                        self.reset_view = true;
                        self.y_min = value;
                    }
                    ui.label(format!("({:.2e})", self.y_min));
                }

                ui.label("Y Max:");
                ui.text_edit_singleline(&mut self.y_max_input);
                if let Ok(value) = self.y_max_input.parse::<f64>() {
                    if self.y_max != value {
                        self.reset_view = true;
                        self.y_max = value;
                    }
                    ui.label(format!("({:.2e})", self.y_max));
                }
            });

        });

        // Bottom panel for current bounds display
        egui::TopBottomPanel::bottom("current_bounds_panel").show(ctx, |ui| {
            ui.heading("Current View Bounds");
            if let Some(bounds) = self.current_bounds {
                // X bounds display
                ui.horizontal(|ui| {
                    ui.strong("X Range:");
                    ui.label(format!("[{:.2e}, {:.2e}]", bounds.min()[0], bounds.max()[0]));
                });

                // Y bounds display
                ui.horizontal(|ui| {
                    ui.strong("Y Range:");
                    ui.label(format!("[{:.2e}, {:.2e}]", bounds.min()[1], bounds.max()[1]));
                });

                // Width and height
                ui.horizontal(|ui| {
                    let x_width = bounds.max()[0] - bounds.min()[0];
                    let y_height = bounds.min()[1] - bounds.max()[1];
                    ui.label(format!("Width: {:.2e}", x_width));
                    ui.label(format!("Height: {:.2e}", y_height));
                    ui.label(format!("Aspect Ratio: {:.2}", x_width / y_height));
                });
            }
        });

        // Central panel for the plot
        egui::CentralPanel::default().show(ctx, |ui| {
            let points = sample_curve_u256(NUM_POINTS);
            let mut plot = Plot::new("plot")
                .default_x_bounds(self.x_min, self.x_max)
                .default_y_bounds(self.y_min, self.y_max)
                .auto_bounds(true);

            if self.reset_view {
                plot = plot.reset();
                self.reset_view = false;
            }

            let plot_response = plot.show(ui, |plot_ui| {
                plot_ui.points(points);
            });

            // Get the current bounds directly from the plot transform
            let transform = plot_response.transform;
            let min_pos = transform.value_from_position(plot_response.response.rect.left_top());
            let max_pos = transform.value_from_position(plot_response.response.rect.right_bottom());
            let bounds = egui_plot::PlotBounds::from_min_max(
                [min_pos.x, min_pos.y],
                [max_pos.x, max_pos.y]
            );
            self.current_bounds = Some(bounds);
        });
    }
}


fn sample_curve_u256(
    num_points: usize,
) -> Points<'static> {


    let line = Points::new("y = f(x)", PlotPoints::from_explicit_callback(
        move |x_f64: f64| {
        if x_f64.is_infinite() {
            return 0.0;
        }
        // Convert x_f64 -> U256
        let x_u256 = f64_to_u256(x_f64, X_RADIX, X_PLACES);
        let y_u256 = plot_fun(x_u256);
        let r = u256_to_f64(y_u256, Y_RADIX, Y_PLACES);
//        println!("x_f64 {:?} x {:?} y {:?} r {:?}", x_f64, x_u256, y_u256, r);
        r
    },
    .., // infinite
    num_points));
    // println!("line_bounds = {:?}", line.bounds());
    line
}
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Plot: U256 fixed-point",
        options,
        Box::new(|_cc| Ok(Box::new(EllipticApp::default()))),
    )
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////


