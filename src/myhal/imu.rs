use ahrs::{Ahrs, Madgwick};
use nalgebra as na;
use core::f64::consts::PI;

pub trait Gyro {
    /// Get the measured angular rates in degrees per second
    fn get_angular_rates(&mut self) -> na::Vector3<f64>;

    // fn get_x(&mut self) -> f64;
    // fn get_y(&mut self) -> f64;
    // fn get_z(&mut self) -> f64;
}

pub trait Accelerometer {
    /// Get the measured accelerations in g's
    fn get_accelerations(&mut self) -> na::Vector3<f64>;

    // fn get_x(&mut self) -> f64;
    // fn get_y(&mut self) -> f64;
    // fn get_z(&mut self) -> f64;
}

// TODO: Magnetometer trait

pub trait Orientation {
    fn get_rotations(&mut self) -> (f64, f64, f64);
}

pub struct IMU<T: Gyro + Accelerometer> {
    pub imu: T,
    filter: Madgwick<f64>,
}

impl<T: Gyro + Accelerometer> IMU<T> {
    pub fn new(imu: T) -> Self {
        Self {
            imu,
            filter: Madgwick::new(0.01, 0.2),
        }
    }

    pub fn update(&mut self) {
        let angular_rates = self.get_angular_rates() * (PI / 180.0);
        let accelerations = self.get_accelerations() * 9.8;
        self.filter.update_imu(&angular_rates, &accelerations).ok();
    }
}

impl<T: Gyro + Accelerometer> Orientation for IMU<T> {
    fn get_rotations(&mut self) -> (f64, f64, f64) {
        self.filter.quat.euler_angles()
    }
}

impl<T: Gyro + Accelerometer> Gyro for IMU<T> {
    fn get_angular_rates(&mut self) -> na::Vector3<f64> {
        self.imu.get_angular_rates()
    }
}

impl<T: Gyro + Accelerometer> Accelerometer for IMU<T> {
    fn get_accelerations(&mut self) -> na::Vector3<f64> {
        self.imu.get_accelerations()
    }
}
