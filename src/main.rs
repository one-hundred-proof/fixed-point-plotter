use eframe::egui::{self, Color32, Pos2};
use egui_plot::{Line, Plot, PlotBounds, PlotItem, PlotPoints, PlotUi, Points};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitive_types::U256;

const DECIMALS: u32 = 18;


pub struct EllipticApp {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl Default for EllipticApp {
    fn default() -> Self {
        let (x_min, x_max, y_min, y_max) = (100e15, 980e15, 0.0, 1.5);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}
impl eframe::App for EllipticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot (U256 scaled)");

            let points = sample_curve_u256(self.x_min, self.x_max, 10000);
            Plot::new("plot")
                .default_x_bounds(self.x_min, self.x_max)
                .default_y_bounds(self.y_min, self.y_max)
                .auto_bounds(true) // turn auto bounds back on after setting defaults
                .show(ui, |plot_ui| {
                    plot_ui.points(points);
                });
        });
    }
}

// Converts a U256 fixed-point number to f64 with `decimals` fractional digits
fn u256_to_f64_decimal(value: U256, decimals: u32) -> f64 {
    let factor = 10u128.pow(decimals);
    let mut buf = [0u8; 32];
    buf = value.to_big_endian();
    let big_value = BigUint::from_bytes_be(&buf);
    let int_part = (&big_value / factor).to_f64().unwrap_or(0.0);
    let frac_part = (&big_value % factor).to_f64().unwrap_or(0.0) / factor as f64;
    int_part + frac_part
}

pub fn f64_to_u256_decimal(value: f64, decimals: u32) -> U256 {
    let factor = 10f64.powi(decimals as i32); // 1e18
    let scaled = value * factor;
    // Clamp negative values to 0 since U256 can't represent them
    if scaled.is_sign_negative() {
        U256::zero()
    } else {
        U256::from(scaled as u128)
    }
}

fn mul(x: U256, y: U256) -> U256 {
    x.overflowing_mul(y).0 / U256::from(10u128.pow(DECIMALS))
}

fn div(x: U256, y: U256) -> U256 {
    x.overflowing_mul(U256::from(10u128.pow(DECIMALS))).0 / y
}


fn plot_fun(x: U256) -> U256 {
    if x > U256::from(0) {
        mul(x, div(U256::from(10u128.pow(DECIMALS)), x))
    } else {
        U256::from(0)
    }
}

fn sample_curve_u256(
    x_min: f64,
    x_max: f64,
    num_points: usize,
) -> Points<'static> {

    let xrange = x_min..=x_max;

    let line = Points::new("y = f(x)", PlotPoints::from_explicit_callback(
        move |x_f64: f64| {
        // Convert x_f64 -> U256
        let x_u256 = f64_to_u256_decimal(x_f64, DECIMALS);
//        println!("x_u256 = {:?}", x_u256);
        let y_u256 = plot_fun(x_u256);
//        println!("x = {:?} , y = {:?}", x_u256, y_u256 );
        u256_to_f64_decimal(y_u256, DECIMALS)
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
