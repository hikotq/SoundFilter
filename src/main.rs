//! A demonstration of recording to wav an input stream
//!
//! Audio from the default input device is stored in to a wav file
extern crate num;
extern crate portaudio;
extern crate rustfft;

mod fft;
mod filter;

use std::thread::sleep;
use std::time::{Duration, Instant};
extern crate hound;
use hound::WavWriter;
use std::fs;
use std::io;
use std::io::{BufWriter, Write};
use std::mem;
use std::sync::{Arc, Mutex};

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 7;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 32368;
const TABLE_SIZE: usize = 200;

fn main() {
    const CHANNELS: i32 = 2;
    const SAMPLE_RATE: f64 = 44_100.0;
    const FRAMES: u32 = 256;
    let audio_port = match open_audio_port() {
        Ok(port) => port,
        Err(error) => panic!(String::from(error)),
    };
    let input_index = match get_input_device_index(&audio_port) {
        Ok(index) => index,
        Err(error) => panic!(String::from(error)),
    };
    let input_settings =
        match audio_port.default_input_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER) {
            Ok(settings) => settings,
            Err(error) => panic!(),
        };
    let mut wav_writer = match get_wav_writer("recorded.wav", CHANNELS, SAMPLE_RATE) {
        Ok(writer) => writer,
        Err(error) => panic!(error),
    };
    //let mut f = BufWriter::new(fs::File::create("original.dump").unwrap());
    {
        let mut filter = filter::SoundFilter::new(FRAMES_PER_BUFFER as usize, SAMPLE_RATE as usize);
        let callback = move |portaudio::InputStreamCallbackArgs { buffer, .. }| {
            let filtered_buffer = filter
                .fft(buffer)
                .highpass_filter(150.0)
                .lowpass_filter(3000.0)
                .ifft();
            //let filtered_buffer = buffer;
            //let mut o = "".to_string();

            for &sample in filtered_buffer.iter() {
                wav_writer.write_sample(sample).ok();
            }
            //f.write(o.as_bytes()).unwrap();
            portaudio::Continue
        };
        // Construct a stream with input and output sample types of f32.
        let mut stream = match audio_port.open_non_blocking_stream(input_settings, callback) {
            Ok(strm) => strm,
            Err(error) => panic!(error.to_string()),
        };
        match stream.start() {
            Ok(_) => {}
            Err(error) => panic!(error.to_string()),
        };
        let start = Instant::now();
        let time_to_wait = &(10 as u64);
        while start.elapsed().as_secs().lt(time_to_wait) {
            sleep(Duration::new(1, 0));
            println!("{}[s] passed", start.elapsed().as_secs());
        }
        match close_stream(stream) {
            Ok(_) => {}
            Err(error) => panic!(error),
        };
    }
}
fn get_wav_writer(
    path: &'static str,
    channels: i32,
    sample_rate: f64,
) -> Result<WavWriter<io::BufWriter<fs::File>>, String> {
    let spec = wav_spec(channels, sample_rate);
    match hound::WavWriter::create(path, spec) {
        Ok(writer) => Ok(writer),
        Err(error) => Err(String::from(format!("{}", error))),
    }
}
fn wav_spec(channels: i32, sample_rate: f64) -> hound::WavSpec {
    hound::WavSpec {
        channels: channels as _,
        sample_rate: sample_rate as _,
        bits_per_sample: (mem::size_of::<f32>() * 8) as _,
        sample_format: hound::SampleFormat::Float,
    }
}
fn close_stream(
    mut stream: portaudio::Stream<portaudio::NonBlocking, portaudio::Input<f32>>,
) -> Result<String, String> {
    match stream.stop() {
        Ok(_) => Ok(String::from("Stream closed")),
        Err(error) => Err(error.to_string()),
    }
}
fn open_audio_port() -> Result<portaudio::PortAudio, String> {
    portaudio::PortAudio::new().or_else(|error| Err(String::from(format!("{}", error))))
}
fn get_input_device_index(
    audio_port: &portaudio::PortAudio,
) -> Result<portaudio::DeviceIndex, String> {
    audio_port
        .default_input_device()
        .or_else(|error| Err(String::from(format!("{}", error))))
}
fn get_input_latency(
    audio_port: &portaudio::PortAudio,
    input_index: portaudio::DeviceIndex,
) -> Result<f64, String> {
    let input_device_information = audio_port
        .device_info(input_index)
        .or_else(|error| Err(String::from(format!("{}", error))));
    Ok(input_device_information.unwrap().default_low_input_latency)
}
fn get_input_stream_parameters(
    input_index: portaudio::DeviceIndex,
    latency: f64,
    channels: i32,
) -> Result<portaudio::StreamParameters<f32>, String> {
    const INTERLEAVED: bool = true;
    Ok(portaudio::StreamParameters::<f32>::new(
        input_index,
        channels,
        INTERLEAVED,
        latency,
    ))
}
fn get_input_settings(
    input_index: portaudio::DeviceIndex,
    audio_port: &portaudio::PortAudio,
    sample_rate: f64,
    frames: u32,
    channels: i32,
) -> Result<portaudio::InputStreamSettings<f32>, String> {
    Ok(portaudio::InputStreamSettings::new(
        (get_input_stream_parameters(
            input_index,
            (get_input_latency(&audio_port, input_index))?,
            channels,
        ))?,
        sample_rate,
        frames,
    ))
}
