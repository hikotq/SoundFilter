use fft::{fft, ifft};
use num::{Float, ToPrimitive};
use rustfft::num_complex::Complex;
use rustfft::FFTnum;
use std::f64::consts::PI;
use std::fmt::Debug;

pub struct SoundFilter<T: FFTnum + Float + ToPrimitive> {
    window: Vec<T>,
    prev_filtered_buf: Vec<T>,
    prev_buf: Vec<Complex<T>>,
    frames_per_buffer: usize,
    resolution: T,
    fft_buffer: Vec<Complex<T>>,
}

impl<T: FFTnum + Float + ToPrimitive + ToString> SoundFilter<T> {
    pub fn new(frames_per_buffer: usize, sample_rate: usize) -> Self {
        let mut window = Vec::with_capacity(4 * frames_per_buffer);
        let hamming = |n: T| -> T {
            T::from(0.54).unwrap()
                - T::from(0.46).unwrap()
                    * ((T::from(2.0 * PI).unwrap() * n)
                        / (T::from(2.0).unwrap() * T::from(2 * frames_per_buffer).unwrap()
                            - T::from(1.0).unwrap()))
                        .cos()
        };
        for i in 0..(4 * frames_per_buffer) {
            window.push(hamming(T::from(i).unwrap()));
        }
        let prev_filtered_buf = vec![T::zero(); 2 * frames_per_buffer];
        let prev_buf = vec![Complex::new(T::zero(), T::zero()); 2 * frames_per_buffer];
        let resolution = T::from(sample_rate).unwrap() / T::from(2 * frames_per_buffer).unwrap();
        let fft_buffer = vec![Complex::new(T::zero(), T::zero()); 4 * frames_per_buffer];
        SoundFilter {
            window: window,
            prev_filtered_buf: prev_filtered_buf,
            prev_buf: prev_buf,
            frames_per_buffer: frames_per_buffer,
            resolution: resolution,
            fft_buffer: fft_buffer,
        }
    }

    pub fn fft(&mut self, buffer: &[T]) -> &mut Self {
        //窓関数をかける
        let mut buf = Vec::new();
        buf.extend_from_slice(&mut self.prev_buf);
        buf.append(&mut buffer
            .iter()
            .map(|v| Complex::new(*v, T::zero()))
            .collect::<Vec<Complex<T>>>());
        Self::dump_vec("buffer", &buf.clone().into_iter().map(|c| c.re).collect());
        for i in 0..buf.len() {
            buf[i].re = buf[i].re * self.window[i];
        }
        for (i, v) in buffer.iter().enumerate() {
            self.prev_buf[i].re = *v;
            self.prev_buf[i].im = T::zero();
        }
        Self::dump_vec(
            "window_buffer",
            &buf.clone().into_iter().map(|c| c.re).collect(),
        );
        fft(&mut buf.as_mut_slice(), &mut self.fft_buffer);
        self
    }

    pub fn highpass_filter(&mut self, fc: T) -> &mut Self {
        let l = self.fft_buffer.len();
        for i in 0..(fc / self.resolution).ceil().to_usize().unwrap() {
            self.fft_buffer[i] = Complex::new(T::zero(), T::zero());
            self.fft_buffer[l - 1 - i] = Complex::new(T::zero(), T::zero());
        }
        self
    }

    pub fn lowpass_filter(&mut self, fc: T) -> &mut Self {
        let l = self.fft_buffer.len();
        for i in (fc / self.resolution).ceil().to_usize().unwrap()
            ..((self.frames_per_buffer * 2) as usize)
        {
            self.fft_buffer[i] = Complex::new(T::zero(), T::zero());
            self.fft_buffer[l - 1 - i] = Complex::new(T::zero(), T::zero());
        }
        self
    }

    pub fn ifft(&mut self) -> Vec<T> {
        let mut buffer = vec![Complex::new(T::zero(), T::zero()); 4 * self.frames_per_buffer];
        ifft(&mut self.fft_buffer, &mut buffer);
        let mut out = Vec::new();
        for i in 0..self.prev_filtered_buf.len() {
            out.push(buffer[i].re + self.prev_filtered_buf[i]);
            self.prev_filtered_buf[i] = buffer[i + self.prev_filtered_buf.len()].re;
        }
        out
    }

    fn dump_vec(s: &str, vec: &Vec<T>) {
        use std::fs;
        use std::io::{BufWriter, Write};
        let mut f = BufWriter::new(fs::File::create(s).unwrap());
        for v in vec.iter() {
            f.write((v.to_string() + "\n").as_bytes()).ok();
        }
    }
}
