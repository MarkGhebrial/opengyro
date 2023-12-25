pub trait Servo {
    fn set_us(&mut self, us: u16);
}

pub struct ServoController<const N: usize, T: Servo> {
    servos: [T; N],
}

impl<const N: usize, T: Servo> ServoController<N, T> {
    pub fn new(servos: [T; N]) -> Self {
        Self { servos }
    }
}
