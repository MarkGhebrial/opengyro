use crate::myhal::imu::AttitudeSensor;
use pid::Pid;

pub struct OrientationFlightMode<IMU: AttitudeSensor> {
    imu: IMU,
    roll_controller: Pid<f64>,
    pitch_controller: Pid<f64>,
}

impl<IMU: AttitudeSensor> OrientationFlightMode<IMU> {
    pub fn new(imu: IMU) -> Self {
        Self {
            imu,
            roll_controller: Pid::new(0, 1),
            pitch_controller: Pid::new(0, 1),
        }
    }
}

impl<IMU: AttitudeSensor> super::FlightMode for OrientationFlightMode<IMU> {
    fn execute(input: super::FlightModeInput) -> super::FlightModeInput {
        todo!()
    }
}