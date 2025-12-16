use eframe::egui::{self, Color32, Slider};
use egui_plot::{Plot, PlotPoints, Points};
use primitive_types::{U256};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod maths;
mod functions;
mod vyper;


use crate::functions::yearn::*;
use crate::maths::*;

const fixed_point_fun: FixedPointFunction = yearn_calc_supply;

pub struct EllipticApp {
    // Sampling bounds (limits on what values can be sampled)
    sampling_x_min: f64,
    sampling_x_max: f64,

    // Display bounds (what we're looking at)
    display_x_min: f64,
    display_x_max: f64,
    display_y_min: f64,
    display_y_max: f64,

    // Input fields
    sampling_x_min_input: String,
    sampling_x_max_input: String,
    display_x_min_input: String,
    display_x_max_input: String,
    display_y_min_input: String,
    display_y_max_input: String,

    current_bounds: Option<egui_plot::PlotBounds>,
    reset_view: bool,
    num_points: usize,
    error_message: Option<String>,
    last_error_x: Option<f64>,
    fps: f32,
    frame_times: Vec<f64>,
    last_frame_time: std::time::Instant,
}

impl Default for EllipticApp {
    fn default() -> Self {
        /* These bounds must be pre-divided by radix^places */
        let (xb, yb) = (fixed_point_fun.x_bounds, fixed_point_fun.y_bounds);
        let (x_min, x_max, y_min, y_max) = (xb.min, xb.max, yb.min, yb.max);
        Self {
            // Initialize sampling bounds
            sampling_x_min: x_min,
            sampling_x_max: x_max,

            // Initialize display bounds
            display_x_min: x_min,
            display_x_max: x_max,
            display_y_min: y_min,
            display_y_max: y_max,

            // Initialize input fields
            sampling_x_min_input: x_min.to_string(),
            sampling_x_max_input: x_max.to_string(),
            display_x_min_input: x_min.to_string(),
            display_x_max_input: x_max.to_string(),
            display_y_min_input: y_min.to_string(),
            display_y_max_input: y_max.to_string(),

            current_bounds: None,
            reset_view: false,
            num_points: fixed_point_fun.num_points.default,
            error_message: None,
            last_error_x: None,
            fps: 0.0,
            frame_times: Vec::with_capacity(100),
            last_frame_time: std::time::Instant::now(),
        }
    }
}

impl eframe::App for EllipticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate FPS
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;

        self.frame_times.push(frame_time);
        if self.frame_times.len() > 100 {
            self.frame_times.remove(0);
        }

        let avg_frame_time: f64 = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        self.fps = (1.0 / avg_frame_time) as f32;

        // Request continuous repainting to update FPS
        ctx.request_repaint();
        egui::TopBottomPanel::top("input_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Plot (U256 scaled)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!("FPS: {:.1}", self.fps));
                });
            });

            // Add controls for adjusting sampling bounds
            ui.collapsing("Sampling Bounds", |ui| {
                ui.label("These bounds limit the range of x values that can be sampled.");

                ui.horizontal(|ui| {
                    ui.label("X Min:");
                    ui.label(format!("{:.2e}", self.sampling_x_min));
                    ui.text_edit_singleline(&mut self.sampling_x_min_input);

                    ui.label("X Max:");
                    ui.label(format!("{:.2e}", self.sampling_x_max));
                    ui.text_edit_singleline(&mut self.sampling_x_max_input);
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.sampling_x_max_input.parse::<f64>() {
                            self.sampling_x_max = value;
                        }
                    }
                });
            });

            // Add controls for adjusting display bounds
            ui.collapsing("Display Bounds", |ui| {
                ui.label("These bounds control what part of the function is displayed.");

                ui.horizontal(|ui| {
                    ui.label("X Min:");
                    ui.label(format!("{:.2e}", self.display_x_min));
                    ui.text_edit_singleline(&mut self.display_x_min_input);
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.display_x_min_input.parse::<f64>() {
                            if self.display_x_min != value {
                                self.display_x_min = value;
                                self.reset_view = true;
                            }
                        }
                    }

                    ui.label("X Max:");
                    ui.label(format!("{:.2e}", self.display_x_max));
                    ui.text_edit_singleline(&mut self.display_x_max_input);
                });

                ui.horizontal(|ui| {
                    ui.label("Y Min:");
                    ui.label(format!("{:.2e}", self.display_y_min));
                    ui.text_edit_singleline(&mut self.display_y_min_input);

                    ui.label("Y Max:");
                    ui.label(format!("{:.2e}", self.display_y_max));
                    ui.text_edit_singleline(&mut self.display_y_max_input);
                });

                if ui.button("Set").clicked() {
                        if let (Ok(y_min_val), Ok(y_max_val), Ok(x_min_val), Ok(x_max_val)) =
                               (self.display_y_min_input.parse::<f64>(),
                                self.display_y_max_input.parse::<f64>(),
                                self.display_x_min_input.parse::<f64>(),
                                self.display_x_max_input.parse::<f64>(),)
                                {
                            self.display_y_min = y_min_val;
                            self.display_y_max = y_max_val;
                            self.display_x_min = x_min_val;
                            self.display_x_max = x_max_val;
                            self.reset_view = true;
                        }
                    }

                if ui.button("Reset to Sampling Bounds").clicked() {
                    self.display_x_min = self.sampling_x_min;
                    self.display_x_max = self.sampling_x_max;
                    self.display_x_min_input = self.display_x_min.to_string();
                    self.display_x_max_input = self.display_x_max.to_string();
                    self.reset_view = true;
                }
            });

            // Add slider for number of points
            ui.horizontal(|ui| {
                let np = fixed_point_fun.num_points;
                ui.label("Number of points:");
                ui.add(Slider::new(&mut self.num_points, np.min..=np.max)
                    .logarithmic(true)
                    .text("points"));
            });

            ui.horizontal(|ui| {
                if let Some(error_msg) = &self.error_message {
                    ui.label(egui::RichText::new("Error:").color(egui::Color32::RED).strong());
                    ui.label(error_msg);
                    if let Some(x) = self.last_error_x {
                        ui.label(format!("at x ≈ {:.6e}", x));
                    }
                }

            });


            // Display error message if any
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
            let mut was_reset = false;
            if let None = self.current_bounds { was_reset = true }

            let mut plot = Plot::new("plot")
                .default_x_bounds(self.display_x_min, self.display_x_max)
                .default_y_bounds(self.display_y_min, self.display_y_max)
                .auto_bounds(true);

            // Get the current x bounds from the plot if available, otherwise use display bounds
            let sample_x_min = if let Some(bounds) = self.current_bounds {
                bounds.min()[0].max(self.sampling_x_min).min(self.sampling_x_max)
            } else {
                self.display_x_min.max(self.sampling_x_min).min(self.sampling_x_max)
            };

            let sample_x_max = if let Some(bounds) = self.current_bounds {
                bounds.max()[0].max(self.sampling_x_min).min(self.sampling_x_max)
            } else {
                self.display_x_max.max(self.sampling_x_min).min(self.sampling_x_max)
            };

            // Sample the curve with panic handling using the current view bounds for x
            let (points, errorPoints) = sample_curve_u256_safe(self.num_points, sample_x_min, sample_x_max);

            // Ensure y_min is always less than y_max
            if self.display_y_min > self.display_y_max {
                std::mem::swap(&mut self.display_y_min, &mut self.display_y_max);
            }
            if self.reset_view {
                plot = plot.reset();
                self.reset_view = false;
                was_reset = true;
            }

            let plot_response = plot.show(ui, |plot_ui| {
                plot_ui.points(points);
                plot_ui.points(errorPoints);
                if was_reset {
                    plot_ui.set_plot_bounds_x(self.display_x_min..=self.display_x_max);
                    plot_ui.set_plot_bounds_y(self.display_y_min..=self.display_y_max);
                }
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


/// Safely sample the curve with panic handling. An x values for which the function reverts will be pushed into a second set of points called `errorPoints`
/// These can be plotted in a different colour
fn sample_curve_u256_safe(
    num_points: usize,
    x_min: f64,
    x_max: f64,
) -> (Points<'static>, Points<'static>) {
    // Create a thread-safe counter to track which x value caused a panic
    let current_x_index = Arc::new(AtomicUsize::new(0));

    // Create a vector to store x values for each point
    let x_values: Vec<f64> = (0..num_points)
        .map(|i| {
            let t = i as f64 / (num_points - 1) as f64;
            x_min + t * (x_max - x_min)
        })
        .collect();

    // Set up panic hook
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // Silent hook

    // Separate points that produce a value from those that panic/revert
    let mut points_vec = Vec::with_capacity(num_points);

    let mut error_points_vec = Vec::with_capacity(num_points);

    for (i, &x) in x_values.iter().enumerate() {
        // Update the current index being processed
        current_x_index.store(i, Ordering::SeqCst);

        if x.is_infinite() {
            points_vec.push([x, 0.0]);
            continue;
        }

        // Convert x_f64 -> U256
        let (xb, yb) = (fixed_point_fun.x_bounds, fixed_point_fun.y_bounds);
        let x_u256 = f64_to_u256(x, xb.radix, xb.places);
        let result_y = panic::catch_unwind(AssertUnwindSafe(|| { (fixed_point_fun.fun)(x_u256) }));
        match result_y {
            Ok(y_u256) => {
                let y = u256_to_f64(y_u256, yb.radix, yb.places);
                points_vec.push([x, y]);
            }
            Err(_) => {

                error_points_vec.push([x,0.0]);
            }
        }
    }

    // Restore the original panic hook
    panic::set_hook(old_hook);


    let mut points = Points::new("y = f(x)", PlotPoints::new(points_vec));
    points = points.color(Color32::DARK_BLUE);
    let mut error_points = Points::new("error points", PlotPoints::new(error_points_vec));
    error_points = error_points.color(Color32::RED);
    (points, error_points)
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Fixed point plotter",
        options,
        Box::new(|_cc| Ok(Box::new(EllipticApp::default()))),
    )
}

