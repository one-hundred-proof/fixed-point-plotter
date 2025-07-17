use eframe::egui::{self, Color32, Pos2};
use egui_plot::{Line, Plot, PlotBounds, PlotItem, PlotPoints};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitive_types::U256;

const DECIMALS: u32 = 18;

#[derive(Debug)]
pub struct EllipticApp {
    x_min: U256,
    x_max: U256,
    y_min: U256,
    y_max: U256,
}

impl Default for EllipticApp {
    fn default() -> Self {
        Self {
            x_min: U256::from_dec_str("100000000000000000000000000000000000").unwrap(),
            x_max: U256::from_dec_str("980000000000000000000000000000000000").unwrap(),
            y_min: U256::from_dec_str("0").unwrap(),
            y_max: U256::from_dec_str("1500000000000000000").unwrap(), // 1.5e18
        }
    }
}
impl eframe::App for EllipticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot of y = x² (U256 scaled)");

            let scroll_delta = ctx.input(|i| i.raw_scroll_delta);
            let pointer_pos: Option<Pos2> = ctx.input(|i| i.pointer.hover_pos());
            let mouse_in_bounds = pointer_pos.map_or(false, |pos| ui.min_rect().contains(pos));

            if mouse_in_bounds && scroll_delta != egui::Vec2::ZERO {
                // Adjust bounds with zoom
                let zoom_factor = if scroll_delta.y > 0.0 { 0.9 } else { 1.1 };
                let zoom_fp = U256::from((zoom_factor * 1_000_000.0) as u128); // fixed-point zoom factor
                let one_million = U256::from(1_000_000u128);

                let x_range = self.x_max - self.x_min;
                let x_mid = self.x_min + x_range / 2;

                let y_range = self.y_max - self.y_min;
                let y_mid = self.y_min + y_range / 2;

                // Zoom range directly (as fixed-point): x_range * zoom_fp / 1_000_000
                let new_x_range = x_range.saturating_mul(zoom_fp) / one_million;
                let new_y_range = y_range.saturating_mul(zoom_fp) / one_million;

                // Half the new range
                let new_x_half = new_x_range / 2;
                let new_y_half = new_y_range / 2;

                // New bounds
                self.x_min = x_mid.saturating_sub(new_x_half);
                self.x_max = x_mid.saturating_add(new_x_half);
                self.y_min = y_mid.saturating_sub(new_y_half);
                self.y_max = y_mid.saturating_add(new_y_half);

            }
            println!("bounds = {:?}", self);

            let line = sample_curve_u256(self.x_min, self.x_max, 10000);

            Plot::new("u256_plot")
                .legend(egui_plot::Legend::default())
                .data_aspect(0.0)
                .include_x(u256_to_f64_decimal(self.x_min, DECIMALS))
                .include_x(u256_to_f64_decimal(self.x_max, DECIMALS))
                .include_y(u256_to_f64_decimal(self.y_min, DECIMALS))
                .include_y(u256_to_f64_decimal(self.y_max, DECIMALS))
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
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
    let factor = 10f64.powi(decimals as i32);
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
    x_min: U256,
    x_max: U256,
    num_points: usize,
) -> Line<'static> {

    let line = Line::new("y = f(x)", PlotPoints::from_explicit_callback(
    move |x_f64: f64| {
        // Convert x_f64 -> U256
        let x_u256 = f64_to_u256_decimal(x_f64, DECIMALS);
        let y_u256 = plot_fun(x_u256);
        // println!("x = {:?} , y = {:?}", x_u256, y_u256 );
        u256_to_f64_decimal(y_u256, DECIMALS)
    },
    u256_to_f64_decimal(x_min, DECIMALS)..=u256_to_f64_decimal(x_max, DECIMALS),
    num_points));
    // println!("line_bounds = {:?}", line.bounds());
    line


}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Plot: y = x² (U256 fixed-point)",
        options,
        Box::new(|_cc| Ok(Box::new(EllipticApp::default()))),
    )
}
