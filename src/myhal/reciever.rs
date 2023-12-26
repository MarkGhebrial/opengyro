/// A trait for reciever implementations.
pub trait Reciever<const NUM_CHANNELS: usize> {
    /// Returns true if there are new channel positions to read
    fn has_new_data(&self) -> bool;

    /// Return an array of the PWM pulse width for each channel (in microseconds)
    ///
    /// Calling this function will make `has_new_data()` return false until another
    /// packet is recieved from the transmitter
    fn get_channels(&mut self) -> [u16; NUM_CHANNELS];

    /// Return the minimum microsecond value for the specified channel
    fn get_channel_min(&self, ch_num: usize) -> u16;

    /// Return the maximum microsecond value for the specified channel
    fn get_channel_max(&self, ch_num: usize) -> u16;

    /// Returns true if the reciever has lost connection with the transmitter
    /// and has entered failsafe mode
    fn is_in_failsafe(&self) -> bool;
}
