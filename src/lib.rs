#![no_std]

use dasp::{
    Signal,
    signal::{
        self, ConstHz, Saw, Sine, Square,
        bus::{Bus, Output, SignalBus},
    },
};
use libm::sqrtf;
use microfft::real::rfft_1024;

const FFT_BUFFER_SIZE: usize = 1024;

pub struct Oscillator<S: Signal<Frame = f64>> {
    freq: Option<f64>,
    sample_rate: Option<f64>,
    pub bus: Bus<S>,
    pub main_send: Output<S>,
    fft_send: Output<S>,
    fft_buffer: [f32; FFT_BUFFER_SIZE],
    pub fft_cursor: usize,
}

impl<S: Signal<Frame = f64>> Oscillator<S> {
    pub fn new(freq: Option<f64>, sample_rate: Option<f64>, signal: S) -> Self {
        let bus = signal.bus();
        let fft_send = bus.send();
        let main_send = bus.send();
        Self {
            freq,
            sample_rate,
            bus,
            main_send,
            fft_send,
            fft_buffer: [0.0; FFT_BUFFER_SIZE],
            fft_cursor: 0,
        }
    }

    pub fn real_fft_1024(&mut self) -> [f32; 512] {
        // It might make sense to make this function return an option that is only Some when the
        // fft_cursor is 0. This may also avoid the need for a copy of the buffer, but only if the
        // buffer is ever consumed once every time since it may be modified.
        // Reorder ring buffer so oldest sample is first
        let mut ordered = [0.0f32; FFT_BUFFER_SIZE];
        let (a, b) = self.fft_buffer.split_at(self.fft_cursor);
        ordered[..b.len()].copy_from_slice(b);
        ordered[b.len()..].copy_from_slice(a);
        // dasp comes with a hann window function that gets applied to a signal but for whatever
        // reason that did not work and broke the FFT. Doing it manually seems to be fine.
        // Maybe switch this out in the future for something a little faster.
        (0..FFT_BUFFER_SIZE).for_each(|i| {
            let hann = 0.5
                * (1.0 - (2.0 * core::f32::consts::PI * i as f32 / FFT_BUFFER_SIZE as f32).cos());
            ordered[i] *= hann;
        });
        let spectrum = rfft_1024(&mut ordered);
        spectrum.map(|c| sqrtf(c.re * c.re + c.im * c.im))
    }

    pub fn tick(&mut self) -> f32 {
        let audio_sample = self.main_send.next() as f32;
        let fft_sample = self.fft_send.next() as f32;

        self.fft_buffer[self.fft_cursor] = fft_sample;
        self.fft_cursor = (self.fft_cursor + 1) % FFT_BUFFER_SIZE;

        audio_sample
    }
}

impl Oscillator<Square<ConstHz>> {
    pub fn new_square(freq: f64, sample_rate: f64) -> Self {
        let bus = signal::rate(sample_rate).const_hz(freq).square().bus();
        let fft_send = bus.send();
        let main_send = bus.send();
        Self {
            freq: Some(freq),
            sample_rate: Some(sample_rate),
            bus,
            main_send,
            fft_send,
            fft_buffer: [0.0; 1024],
            fft_cursor: 0,
        }
    }
}

impl Oscillator<Sine<ConstHz>> {
    pub fn new_sine(freq: f64, sample_rate: f64) -> Self {
        let bus = signal::rate(sample_rate).const_hz(freq).sine().bus();
        let fft_send = bus.send();
        let main_send = bus.send();
        Self {
            freq: Some(freq),
            sample_rate: Some(sample_rate),
            bus,
            main_send,
            fft_send,
            fft_buffer: [0.0; 1024],
            fft_cursor: 0,
        }
    }
}

impl Oscillator<Saw<ConstHz>> {
    pub fn new_saw(freq: f64, sample_rate: f64) -> Self {
        let bus = signal::rate(sample_rate).const_hz(freq).saw().bus();
        let fft_send = bus.send();
        let main_send = bus.send();
        Self {
            freq: Some(freq),
            sample_rate: Some(sample_rate),
            bus,
            main_send,
            fft_send,
            fft_buffer: [0.0; 1024],
            fft_cursor: 0,
        }
    }
}

pub fn square_oscillator(sample_rate: f64, freq: f64) -> Square<ConstHz> {
    signal::rate(sample_rate).const_hz(freq).square()
}

pub fn sine_oscillator(sample_rate: f64, freq: f64) -> Sine<ConstHz> {
    signal::rate(sample_rate).const_hz(freq).sine()
}

pub fn saw_oscillator(sample_rate: f64, freq: f64) -> Saw<ConstHz> {
    signal::rate(sample_rate).const_hz(freq).saw()
}

// TODO: Custom triangle wave oscilator
