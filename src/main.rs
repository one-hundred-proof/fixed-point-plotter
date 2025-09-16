use eframe::egui::{self};
use egui_plot::{Plot, PlotPoints, Points};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitive_types::{U256, U512};

const NUM_POINTS: usize = 20000;


const X_RADIX: u8   = 10;
const X_PLACES: u32 = 0;
const X_MIN: f64    = 0.0;
const X_MAX: f64    = 5e38;

const Y_RADIX: u8   = 2;
const Y_PLACES: u32 = 128;
const Y_MIN: f64    = 0.0;
const Y_MAX: f64    = 0.6;



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

/*
 *   The function to be plotted
 */
fn loss_of_rewards(x: U256) -> U256 {
    if x < U256::pow(U256::from(2), U256::from(150)) {
        let growth_inside = to_X128(0.5);
        let last_liquidity = U256::from(1);
        let new_liquidity = x * last_liquidity + 1;
        let last_growth_inside_x128 = U256::from(0);

        let last_growth_adjustment =
            full_mul_div(growth_inside - last_growth_inside_x128, last_liquidity, new_liquidity);
        let last_growth_inside_x128_1 = growth_inside - last_growth_adjustment;


        return (growth_inside - last_growth_inside_x128) * last_liquidity - (growth_inside - last_growth_inside_x128_1) * new_liquidity;
    } else {
        return U256::from(0);
    }

}

fn _x_mul_inverse(x: U256) -> U256 {
    if x > U256::from(0) {
        mul(x, div(U256::from(10u128.pow(18)), x))
    } else {
        U256::from(0)
    }
}

const plot_fun: fn(U256) -> U256 = loss_of_rewards;

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

// Converts a U256 fixed-point number to f64 with `decimals` fractional digits
fn u256_to_f64(value: U256, radix: u8, places: u32) -> f64 {
    let factor: BigUint = BigUint::from(radix).pow(places);

    let buf = value.to_big_endian();
    let big_value = BigUint::from_bytes_be(&buf);
    let int_part = (&big_value / &factor).to_f64().unwrap_or(0.0);

    if let Some(factor_64) = factor.to_f64() {
      let frac_part = (&big_value % &factor).to_f64().unwrap_or(0.0) / factor_64;
      int_part + frac_part
    } else {
       0f64
    }
}

pub fn f64_to_u256(value: f64, radix: u8, places: u32) -> U256 {
    let factor = (radix as f64).powi(places as i32);
    let scaled = value * factor;
    // Clamp negative values to 0 since U256 can't represent them
    if scaled.is_sign_negative() {
        U256::zero()
    } else {
        U256::from(scaled as u128)
    }
}

/*
 *  Converts a floating point number to Uniswaps X128 format (Q128.128)
 */
pub fn to_X128(x: f64) -> U256 {
    f64_to_u256(x,2,128)
}

/*
 *  Converts a floating point number to Uniswaps X128 format (Q64.96)
 */
pub fn to_X96(x: f64) -> U256 {
    f64_to_u256(x,2,96)
}

fn mul(x: U256, y: U256) -> U256 {
    x.overflowing_mul(y).0 / U256::from(10u128.pow(18))
}

fn div(x: U256, y: U256) -> U256 {
    x.overflowing_mul(U256::from(10u128.pow(18))).0 / y
}

/*
 * Uses 512-bit arithmetic in intermediate calculations to retain precision
 */

fn full_mul_div(x: U256, y: U256, z: U256) -> U256 {
    let x_512: U512 = U512::from(x);
    let y_512: U512 = U512::from(y);
    let z_512: U512 = U512::from(z);
    return U256::from_little_endian( &((x_512 * y_512) / z_512).to_little_endian()[0..32]);
}


fn sample_curve_u256(
    num_points: usize,
) -> Points<'static> {


    let line = Points::new("y = f(x)", PlotPoints::from_explicit_callback(
        move |x_f64: f64| {
        // Convert x_f64 -> U256
        let x_u256 = f64_to_u256(x_f64, X_RADIX, X_PLACES);
        let y_u256 = plot_fun(x_u256);
        u256_to_f64(y_u256, Y_RADIX, Y_PLACES)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_math_div() -> Result<(), Box<dyn std::error::Error>> {
        let max_256_str = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let max_256 = U256::from_str_radix(max_256_str, 10)?;
        assert_eq!(full_mul_div(max_256, max_256, max_256), max_256);

        let x: U256 = U256::pow(U256::from(2), U256::from(129)) + 30;
        let y: U256 = U256::pow(U256::from(2), U256::from(135)) + 456;
        let z: U256 = U256::pow(U256::from(2), U256::from(50)) - 12345;

        assert_eq!(full_mul_div(x,y,z), U256::from_str_radix("26328072917427972477888272069236239697031062744045588567990936851", 10)?);

        Ok(())
    }

    #[test]
    fn test_plot_fun() -> Result<(), Box<dyn std::error::Error>> {

        assert_eq!(plot_fun(U256::from_str_radix("10", 10)?),                                U256::from(3));
        assert_eq!(plot_fun(U256::from_str_radix("100", 10)?),                               U256::from(80));
        assert_eq!(plot_fun(U256::from_str_radix("1000", 10)?),                              U256::from(256));
        assert_eq!(plot_fun(U256::from_str_radix("10000", 10)?),                             U256::from(2778));
        assert_eq!(plot_fun(U256::from_str_radix("100000", 10)?),                            U256::from(62648));
        assert_eq!(plot_fun(U256::from_str_radix("1000000", 10)?),                           U256::from(329744));

        assert_eq!(plot_fun(U256::pow(U256::from(2), U256::from(127))),
                   U256::from_str_radix("170141183460469231731687303715884105727", 10)?);


        Ok(())
    }
}
