use eframe::egui::{self, Slider};
use egui_plot::{Plot, PlotPoints, Points};
use primitive_types::{U256};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod maths;
mod functions;

use crate::functions::*;
use crate::maths::*;

// Default number of points, will be configurable via UI
const DEFAULT_NUM_POINTS: usize = 5000;
const MIN_NUM_POINTS: usize = 100;
const MAX_NUM_POINTS: usize = 10000;

const X_RADIX: u8   = 10;
const X_PLACES: u32 = 18;
const X_MIN: f64    = 0.0;
const X_MAX: f64    = 1e18;

const Y_RADIX: u8   = 10;
const Y_PLACES: u32 = 18;
const Y_MIN: f64    = 0.0;
const Y_MAX: f64    = 1.0;

const plot_fun: fn(U256) -> U256 = x_mul_inverse;

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
        let (x_min, x_max, y_min, y_max) = (X_MIN, X_MAX, Y_MIN, Y_MAX);
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
            num_points: DEFAULT_NUM_POINTS,
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
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.sampling_x_min_input.parse::<f64>() {
                            self.sampling_x_min = value;
                        }
                    }

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
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.display_x_max_input.parse::<f64>() {
                            if self.display_x_max != value {
                                self.display_x_max = value;
                                self.reset_view = true;
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Y Min:");
                    ui.label(format!("{:.2e}", self.display_y_min));
                    ui.text_edit_singleline(&mut self.display_y_min_input);
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.display_y_min_input.parse::<f64>() {
                            if self.display_y_min != value {
                                self.display_y_min = value;
                                self.reset_view = true;
                            }
                        }
                    }

                    ui.label("Y Max:");
                    ui.label(format!("{:.2e}", self.display_y_max));
                    ui.text_edit_singleline(&mut self.display_y_max_input);
                    if ui.button("Set").clicked() {
                        if let Ok(value) = self.display_y_max_input.parse::<f64>() {
                            if self.display_y_max != value {
                                self.display_y_max = value;
                                self.reset_view = true;
                            }
                        }
                    }
                });

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
                ui.label("Number of points:");
                ui.add(Slider::new(&mut self.num_points, MIN_NUM_POINTS..=MAX_NUM_POINTS)
                    .logarithmic(true)
                    .text("points"));
            });

            // Display error message if any
            if let Some(error_msg) = &self.error_message {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Error:").color(egui::Color32::RED).strong());
                    ui.label(error_msg);
                    if let Some(x) = self.last_error_x {
                        ui.label(format!("at x ≈ {:.6e}", x));
                    }
                });
            }
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
            let (points, error) = sample_curve_u256_safe(self.num_points, sample_x_min, sample_x_max);

            self.error_message = error.map(|(msg, x)| {
                self.last_error_x = Some(x);
                msg
            });
            // Ensure y_min is always less than y_max
            if self.display_y_min > self.display_y_max {
                std::mem::swap(&mut self.display_y_min, &mut self.display_y_max);
            }

            let mut was_reset = false;
            if self.reset_view {
                plot = plot.reset();
                self.reset_view = false;
                was_reset = true;
            }

            let plot_response = plot.show(ui, |plot_ui| {
                plot_ui.points(points);
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


/// Safely sample the curve with panic handling
fn sample_curve_u256_safe(
    num_points: usize,
    x_min: f64,
    x_max: f64,
) -> (Points<'static>, Option<(String, f64)>) {
    // Create a thread-safe counter to track which x value caused a panic
    let current_x_index = Arc::new(AtomicUsize::new(0));
    let current_x_index_clone = current_x_index.clone();

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

    // Try to compute all points
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut points = Vec::with_capacity(num_points);

        for (i, &x) in x_values.iter().enumerate() {
            // Update the current index being processed
            current_x_index.store(i, Ordering::SeqCst);

            if x.is_infinite() {
                points.push([x, 0.0]);
                continue;
            }

            // Convert x_f64 -> U256
            let x_u256 = f64_to_u256(x, X_RADIX, X_PLACES);
            let y_u256 = plot_fun(x_u256);
            let y = u256_to_f64(y_u256, Y_RADIX, Y_PLACES);

            points.push([x, y]);
        }

        PlotPoints::new(points)
    }));

    // Restore the original panic hook
    panic::set_hook(old_hook);

    match result {
        Ok(plot_points) => {
            // No panic occurred
            (Points::new("y = f(x)", plot_points), None)
        }
        Err(e) => {
            // A panic occurred
            let error_index = current_x_index_clone.load(Ordering::SeqCst);
            let error_x = if error_index < x_values.len() {
                x_values[error_index]
            } else {
                0.0 // Fallback
            };

            // Extract panic message if possible
            let error_message = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&'static str>() {
                s.to_string()
            } else {
                "Unknown error".to_string()
            };

            // Create a partial plot with points up to the error
            let partial_points = if error_index > 0 {
                let mut points = Vec::with_capacity(error_index);
                for i in 0..error_index {
                    let x = x_values[i];
                    if x.is_infinite() {
                        points.push([x, 0.0]);
                        continue;
                    }

                    // Safely compute points before the error
                    let x_u256 = f64_to_u256(x, X_RADIX, X_PLACES);
                    let y_u256 = plot_fun(x_u256);
                    let y = u256_to_f64(y_u256, Y_RADIX, Y_PLACES);

                    points.push([x, y]);
                }
                PlotPoints::new(points)
            } else {
                PlotPoints::new(vec![[x_min, 0.0], [x_max, 0.0]])
            };

            (Points::new("y = f(x) (partial)", partial_points), Some((error_message, error_x)))
        }
    }
}

/// Original sampling function (kept for reference)
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
        r
    },
    .., // infinite
    num_points));
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


