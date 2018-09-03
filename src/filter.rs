extern crate hound;

use fft::{fft, ifft};
use num::{Float, ToPrimitive};
use rustfft::num_complex::Complex;
use rustfft::FFTnum;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Debug;

pub trait SoundFilter<T: FFTnum + Float + ToPrimitive> {
    fn do_filtering(&mut self, buffer: &[T]) -> Vec<T>;
}

pub struct NoiseCancelFilter<T: FFTnum + Float + ToPrimitive> {
    window: Vec<T>,
    last_filtered_buf: Vec<T>,
    last_buf: Vec<Complex<T>>,
    frames_per_buffer: usize,
    resolution: T,
    fft_buffer: Vec<Complex<T>>,
    highpass_fc: T,
    lowpass_fc: T,
}

impl<T: FFTnum + Float + ToPrimitive> NoiseCancelFilter<T> {
    pub fn new(
        frames_per_buffer: usize,
        sample_rate: usize,
        highpass_fc: T,
        lowpass_fc: T,
    ) -> Self {
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
        let last_filtered_buf = vec![T::zero(); 2 * frames_per_buffer];
        let last_buf = vec![Complex::new(T::zero(), T::zero()); 2 * frames_per_buffer];
        let resolution = T::from(sample_rate).unwrap() / T::from(2 * frames_per_buffer).unwrap();
        let fft_buffer = vec![Complex::new(T::zero(), T::zero()); 4 * frames_per_buffer];
        NoiseCancelFilter {
            window: window,
            last_filtered_buf: last_filtered_buf,
            last_buf: last_buf,
            frames_per_buffer: frames_per_buffer,
            resolution: resolution,
            fft_buffer: fft_buffer,
            highpass_fc: highpass_fc,
            lowpass_fc: lowpass_fc,
        }
    }

    pub fn fft(&mut self, buffer: &[T]) {
        //窓関数をかける
        let mut buf = Vec::new();
        buf.extend_from_slice(&mut self.last_buf);
        buf.append(
            &mut buffer
                .iter()
                .map(|v| Complex::new(*v, T::zero()))
                .collect::<Vec<Complex<T>>>(),
        );
        //Self::dump_vec("buffer", &buf.clone().into_iter().map(|c| c.re).collect());
        for i in 0..buf.len() {
            buf[i].re = buf[i].re * self.window[i];
        }
        for (i, v) in buffer.iter().enumerate() {
            self.last_buf[i].re = *v;
            self.last_buf[i].im = T::zero();
        }
        //Self::dump_vec(
        //    "window_buffer",
        //    &buf.clone().into_iter().map(|c| c.re).collect(),
        //);
        fft(&mut buf.as_mut_slice(), &mut self.fft_buffer);
    }

    pub fn highpass_filter(&mut self) {
        let l = self.fft_buffer.len();
        for i in 0..(self.highpass_fc / self.resolution)
            .ceil()
            .to_usize()
            .unwrap()
        {
            self.fft_buffer[i] = Complex::new(T::zero(), T::zero());
            self.fft_buffer[l - 1 - i] = Complex::new(T::zero(), T::zero());
        }
    }

    pub fn lowpass_filter(&mut self) {
        let l = self.fft_buffer.len();
        for i in (self.lowpass_fc / self.resolution)
            .ceil()
            .to_usize()
            .unwrap()..((self.frames_per_buffer * 2) as usize)
        {
            self.fft_buffer[i] = Complex::new(T::zero(), T::zero());
            self.fft_buffer[l - 1 - i] = Complex::new(T::zero(), T::zero());
        }
    }

    pub fn ifft(&mut self) -> Vec<T> {
        let mut buffer = vec![Complex::new(T::zero(), T::zero()); 4 * self.frames_per_buffer];
        ifft(&mut self.fft_buffer, &mut buffer);
        let mut out = Vec::new();
        for i in 0..self.last_filtered_buf.len() {
            out.push(buffer[i].re + self.last_filtered_buf[i]);
            self.last_filtered_buf[i] = buffer[i + self.last_filtered_buf.len()].re;
        }
        out
    }
}

impl<T: FFTnum + Float + ToPrimitive> SoundFilter<T> for NoiseCancelFilter<T> {
    fn do_filtering(&mut self, buffer: &[T]) -> Vec<T> {
        self.fft(buffer);
        self.highpass_filter();
        self.lowpass_filter();
        self.ifft()
    }
}

pub struct RevrebFilter<T: FFTnum + Float + ToPrimitive> {
    reverb_buffer: Vec<T>,
    imp_res: Vec<Complex<T>>,
    fft_buffer: Vec<Complex<T>>,
    frames_per_buffer: usize,
}

impl<T: FFTnum + Float + ToPrimitive> RevrebFilter<T> {
    pub fn new(frames_per_buffer: usize, sample_rate: usize, imp_res_file_name: &str) -> Self {
        let ONEOVERSHORTMAX = T::from(3.0517578125e-5).unwrap();
        let mut reader = hound::WavReader::open(imp_res_file_name).unwrap();
        let mut imp_res = Vec::new();
        for data in reader.samples::<i16>() {
            imp_res.push(match data {
                Ok(data) => Complex::new(T::from(data).unwrap() * ONEOVERSHORTMAX, T::zero()),
                _ => Complex::new(T::zero(), T::zero()),
            });
        }

        let block = (T::from(imp_res.len()).unwrap() / T::from(frames_per_buffer * 2).unwrap())
            .ceil()
            .to_usize()
            .unwrap();
        let reverb_buffer_len = {
            let len = frames_per_buffer * 2 * block;
            len
        };
        let reverb_buffer = vec![T::zero(); reverb_buffer_len];
        for _ in 0..(reverb_buffer.len() - imp_res.len()) {
            imp_res.push(Complex::new(T::zero(), T::zero()));
        }
        let fft_buffer = vec![Complex::new(T::zero(), T::zero()); reverb_buffer.len()];

        RevrebFilter {
            reverb_buffer: reverb_buffer,
            imp_res: imp_res,
            fft_buffer: fft_buffer,
            frames_per_buffer: frames_per_buffer,
        }
    }
}

impl<T: FFTnum + Float + ToPrimitive> SoundFilter<T> for RevrebFilter<T> {
    fn do_filtering(&mut self, buffer: &[T]) -> Vec<T> {
        for i in 0..self.reverb_buffer.len() - buffer.len() {
            self.reverb_buffer[i] = self.reverb_buffer[i + buffer.len()];
        }
        let reverb_buffer_len = self.reverb_buffer.len();
        for i in 0..buffer.len() {
            self.reverb_buffer[reverb_buffer_len - buffer.len() + i] = buffer[i];
        }
        let mut tmp_buffer = self
            .reverb_buffer
            .iter()
            .map(|rb| Complex::new(*rb, T::zero()))
            .collect::<Vec<Complex<T>>>();
        let mut imp_res = self.imp_res.clone();
        fft(&mut tmp_buffer, &mut self.fft_buffer);
        fft(&mut imp_res, &mut tmp_buffer);
        for i in 0..self.fft_buffer.len() {
            self.fft_buffer[i] = self.fft_buffer[i] * tmp_buffer[i];
        }
        ifft(&mut self.fft_buffer, &mut tmp_buffer);
        let out = tmp_buffer
            .into_iter()
            .rev()
            .map(|comp| comp.re)
            .take(buffer.len())
            .collect::<Vec<T>>()
            .into_iter()
            .rev()
            .collect();
        out
    }
}
