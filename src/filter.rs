use fft::{fft, ifft};
use num::{Float, ToPrimitive};
use rustfft::num_complex::Complex;
use rustfft::FFTnum;
use std::fmt::Debug;
use std::io::{BufWriter, Write};
use std::io;
use std::fs;
use std::path::Path;

pub fn highpass_filter<T: FFTnum + Float + ToPrimitive + ToString + Debug>(buffer: &[T], fc: T) -> Vec<T> {
    let mut tmp = vec![Complex::new(T::zero(), T::zero()); 2 * ::FRAMES_PER_BUFFER as usize];
    let mut buf = buffer
        .into_iter()
        .map(|v| Complex::new(*v, T::zero()))
        .collect::<Vec<Complex<T>>>();
    let resolution = T::from(::SAMPLE_RATE).unwrap() / T::from(2 *::FRAMES_PER_BUFFER).unwrap();
    fft(&mut buf.as_mut_slice(), &mut tmp);

    
    if !Path::new("original.dump").exists(){
    let mut f = BufWriter::new(fs::File::create("original.dump").unwrap());
    let mut o = "".to_string();
    for b in tmp.iter() {
        let v = b.norm().to_string() + "\n";
        o.push_str(&v);
    }
    
    f.write(o.as_bytes()).unwrap();
    }
    for i in 0..(fc / resolution).ceil().to_usize().unwrap() {
        tmp[i] = Complex::new(T::zero(), T::zero());
        tmp[2 * ::FRAMES_PER_BUFFER as usize - 1 - i] = Complex::new(T::zero(), T::zero());
    }
    
    for i in (T::from(2000.0).unwrap() / resolution).ceil().to_usize().unwrap()..(::FRAMES_PER_BUFFER as usize) {
        tmp[i] = Complex::new(T::zero(), T::zero());
        tmp[2 * ::FRAMES_PER_BUFFER as usize - 1 -i] = Complex::new(T::zero(), T::zero());
    }
    
    if !Path::new("filtered.dump").exists(){
    let mut f = BufWriter::new(fs::File::create("filtered.dump").unwrap());
    let mut o = "".to_string();
    for b in tmp.iter() {
        let v = b.norm().to_string() + "\n";
        o.push_str(&v);
    }
    
    f.write(o.as_bytes()).unwrap();
    }
    ifft(&mut tmp, buf.as_mut_slice());
    
    let mut out = Vec::new();
    for i in 0..buffer.len() {
        out.push(buf[i].re);
    }
    out
}

