use num::Float;
use rustfft::num_complex::Complex;
use rustfft::{FFTnum, FFTplanner};

fn fft_core<T: FFTnum>(input: &mut [Complex<T>], output: &mut [Complex<T>], inverse: bool) {
    let mut planner = FFTplanner::new(inverse);
    let len = input.len();
    let fft = planner.plan_fft(len);
    fft.process(input, output);
}

pub fn fft<T: FFTnum>(input: &mut [Complex<T>], output: &mut [Complex<T>]) {
    fft_core(input, output, false);
}

pub fn ifft<T: FFTnum + Float>(input: &mut [Complex<T>], output: &mut [Complex<T>]) {
    fft_core(input, output, true);
    for v in output.iter_mut() {
        *v = v.unscale(T::from(input.len() as u32).unwrap());
    }
}
