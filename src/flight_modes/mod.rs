mod passthru;
pub use passthru::*;

mod gyro;
pub use gyro::*;

pub struct FlightModeInput {
    roll: f64,
    pitch: f64,
    yaw: f64,
}

pub trait FlightMode {
    fn execute(input: FlightModeInput) -> FlightModeInput;
}