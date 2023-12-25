use crate::myhal::imu::{Accelerometer, Gyro};
use embedded_hal::blocking::i2c;
use icm20948_driver::icm20948::i2c::IcmImu;

use nalgebra as na;

impl<BUS, E> Gyro for IcmImu<BUS>
where
    BUS: i2c::WriteRead<u8, Error = E> + i2c::Write<u8, Error = E>,
{
    fn get_angular_rates(&mut self) -> na::Vector3<f64> {
        let result = self.read_gyro();

        match result {
            Ok(values) => na::Vector3::new(values[0] as f64, values[1] as f64, values[2] as f64),
            Err(_) => na::Vector3::zeros(),
        }
    }
}

impl<BUS, E> Accelerometer for IcmImu<BUS>
where
    BUS: i2c::WriteRead<u8, Error = E> + i2c::Write<u8, Error = E>,
{
    fn get_accelerations(&mut self) -> na::Vector3<f64> {
        let result = self.read_acc();

        match result {
            Ok(values) => na::Vector3::new(values[0] as f64, values[1] as f64, values[2] as f64),
            Err(_) => na::Vector3::zeros(),
        }
    }
}
