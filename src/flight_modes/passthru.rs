use crate::myhal::imu::AttitudeSensor;


pub struct PassThruFlightMode<IMU: AttitudeSensor> {
    imu: IMU
}