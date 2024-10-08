#![no_std]

pub trait EncoderController<const BITS: u8> {
    // This function returns the absolute step count.
    fn steps(&self) -> usize;

    // This function resets the step count to zero.
    fn reset(&mut self);

    // This function returns a mutable reference to store the last step count.
    // It is implemented to save the state and calculate the delta.
    fn last_steps_ref(&mut self) -> &mut usize;

    // MSB_MASK is used to detect overflow and underflow when the most significant bit changes.
    const MSB_MASK: usize = 1 << (BITS - 1);

    // This function returns the delta of the step count since the last time this function was called.
    fn delta(&mut self) -> (isize, isize) {
        let steps = self.steps();
        let last_steps = self.last_steps_ref();
        let mut delta = steps as isize - *last_steps as isize;
        if steps & Self::MSB_MASK != *last_steps & Self::MSB_MASK {
            delta += match steps > *last_steps {
                true => -(1 << BITS), // underflow
                false => 1 << BITS,   // overflow
            }
        };
        *last_steps = steps;
        (delta, steps as isize)
    }
}

#[cfg(test)]
mod tests;
