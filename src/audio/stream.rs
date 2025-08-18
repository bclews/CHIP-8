//! Audio stream implementation using CPAL.
//!
//! This module provides real-time audio streaming capabilities
//! for the Chip-8 emulator audio system.

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Stream, StreamConfig as CpalStreamConfig,
    SampleFormat, SampleRate, BufferSize,
};
use std::sync::{Arc, Mutex};

use crate::error::AudioError;
use super::{AudioSystemResult, buzzer::{AudioBuzzer, BuzzerState}};

/// Configuration for audio streaming.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    
    /// Buffer size for audio streaming.
    pub buffer_size: u32,
    
    /// Sample format (f32, i16, etc.).
    pub sample_format: StreamSampleFormat,
}

/// Supported sample formats for streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamSampleFormat {
    F32,
    I16,
    U16,
}

impl From<StreamSampleFormat> for SampleFormat {
    fn from(format: StreamSampleFormat) -> Self {
        match format {
            StreamSampleFormat::F32 => SampleFormat::F32,
            StreamSampleFormat::I16 => SampleFormat::I16,
            StreamSampleFormat::U16 => SampleFormat::U16,
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono for simplicity
            buffer_size: 1024,
            sample_format: StreamSampleFormat::F32,
        }
    }
}

impl StreamConfig {
    /// Creates a new stream configuration.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the sample rate.
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }
    
    /// Sets the number of channels.
    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }
    
    /// Sets the buffer size.
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
    
    /// Sets the sample format.
    pub fn with_sample_format(mut self, format: StreamSampleFormat) -> Self {
        self.sample_format = format;
        self
    }
    
    /// Creates a low-latency configuration.
    pub fn low_latency() -> Self {
        Self::new()
            .with_buffer_size(256)
            .with_sample_rate(48000)
    }
    
    /// Creates a high-quality configuration.
    pub fn high_quality() -> Self {
        Self::new()
            .with_buffer_size(2048)
            .with_sample_rate(96000)
            .with_channels(2) // Stereo
    }
}

/// Audio stream for real-time audio output.
pub struct AudioStream {
    /// The CPAL audio stream.
    stream: Stream,
    
    /// Configuration for the stream.
    config: StreamConfig,
    
    /// The audio device being used.
    device: Device,
    
    /// Whether the stream is currently playing.
    is_playing: bool,
}

impl AudioStream {
    /// Creates a new audio stream.
    pub fn new(config: StreamConfig, buzzer: &mut AudioBuzzer) -> AudioSystemResult<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or(AudioError::DeviceNotAvailable)?;
        
        let stream = Self::create_stream(&device, &config, buzzer)?;
        
        Ok(Self {
            stream,
            config,
            device,
            is_playing: false,
        })
    }
    
    /// Creates a stream with automatic device detection.
    pub fn auto_detect(buzzer: &mut AudioBuzzer) -> AudioSystemResult<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or(AudioError::DeviceNotAvailable)?;
        
        let supported_config = device.default_output_config()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;
        
        let config = StreamConfig {
            sample_rate: supported_config.sample_rate().0,
            channels: supported_config.channels(),
            buffer_size: 1024, // Default buffer size
            sample_format: match supported_config.sample_format() {
                SampleFormat::F32 => StreamSampleFormat::F32,
                SampleFormat::I16 => StreamSampleFormat::I16,
                SampleFormat::U16 => StreamSampleFormat::U16,
                _ => StreamSampleFormat::F32, // Fallback
            },
        };
        
        Self::new(config, buzzer)
    }
    
    /// Starts the audio stream.
    pub fn start(&mut self) -> AudioSystemResult<()> {
        self.stream.play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;
        self.is_playing = true;
        Ok(())
    }
    
    /// Pauses the audio stream.
    pub fn pause(&mut self) -> AudioSystemResult<()> {
        self.stream.pause()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;
        self.is_playing = false;
        Ok(())
    }
    
    /// Checks if the stream is playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    
    /// Gets the stream configuration.
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }
    
    /// Gets the audio device information.
    pub fn device_name(&self) -> Result<String, AudioError> {
        self.device.name()
            .map_err(|e| AudioError::StreamError(e.to_string()))
    }
    
    /// Creates the CPAL stream with the appropriate sample format.
    fn create_stream(device: &Device, config: &StreamConfig, buzzer: &AudioBuzzer) -> AudioSystemResult<Stream> {
        let sample_rate = SampleRate(config.sample_rate);
        let channels = config.channels;
        let buffer_size = BufferSize::Fixed(config.buffer_size);
        
        let cpal_config = CpalStreamConfig {
            channels,
            sample_rate,
            buffer_size,
        };
        
        let buzzer_state = buzzer.get_state();
        let sample_rate_f32 = config.sample_rate as f32;
        
        match config.sample_format {
            StreamSampleFormat::F32 => {
                let stream = device.build_output_stream(
                    &cpal_config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        Self::fill_f32_buffer(data, &buzzer_state, sample_rate_f32, channels);
                    },
                    move |err| log::error!("Audio stream error: {}", err),
                    None,
                ).map_err(|e| AudioError::StreamError(e.to_string()))?;
                
                Ok(stream)
            }
            StreamSampleFormat::I16 => {
                let stream = device.build_output_stream(
                    &cpal_config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        Self::fill_i16_buffer(data, &buzzer_state, sample_rate_f32, channels);
                    },
                    move |err| log::error!("Audio stream error: {}", err),
                    None,
                ).map_err(|e| AudioError::StreamError(e.to_string()))?;
                
                Ok(stream)
            }
            StreamSampleFormat::U16 => {
                let stream = device.build_output_stream(
                    &cpal_config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        Self::fill_u16_buffer(data, &buzzer_state, sample_rate_f32, channels);
                    },
                    move |err| log::error!("Audio stream error: {}", err),
                    None,
                ).map_err(|e| AudioError::StreamError(e.to_string()))?;
                
                Ok(stream)
            }
        }
    }
    
    /// Fills an f32 audio buffer.
    fn fill_f32_buffer(
        data: &mut [f32],
        buzzer_state: &Arc<Mutex<BuzzerState>>,
        sample_rate: f32,
        channels: u16,
    ) {
        if let Ok(mut state) = buzzer_state.lock() {
            for frame in data.chunks_mut(channels as usize) {
                let sample = if state.playing {
                    let waveform_sample = (state.phase * 2.0 * std::f32::consts::PI).sin();
                    state.phase += state.frequency / sample_rate;
                    
                    if state.phase >= 1.0 {
                        state.phase -= 1.0;
                    }
                    
                    waveform_sample * state.volume
                } else {
                    0.0
                };
                
                // Fill all channels with the same sample (mono -> stereo duplication)
                for channel_sample in frame {
                    *channel_sample = sample;
                }
            }
        } else {
            // If we can't lock the state, output silence
            data.fill(0.0);
        }
    }
    
    /// Fills an i16 audio buffer.
    fn fill_i16_buffer(
        data: &mut [i16],
        buzzer_state: &Arc<Mutex<BuzzerState>>,
        sample_rate: f32,
        channels: u16,
    ) {
        if let Ok(mut state) = buzzer_state.lock() {
            for frame in data.chunks_mut(channels as usize) {
                let sample = if state.playing {
                    let waveform_sample = (state.phase * 2.0 * std::f32::consts::PI).sin();
                    state.phase += state.frequency / sample_rate;
                    
                    if state.phase >= 1.0 {
                        state.phase -= 1.0;
                    }
                    
                    let float_sample = waveform_sample * state.volume;
                    (float_sample * i16::MAX as f32) as i16
                } else {
                    0
                };
                
                for channel_sample in frame {
                    *channel_sample = sample;
                }
            }
        } else {
            data.fill(0);
        }
    }
    
    /// Fills a u16 audio buffer.
    fn fill_u16_buffer(
        data: &mut [u16],
        buzzer_state: &Arc<Mutex<BuzzerState>>,
        sample_rate: f32,
        channels: u16,
    ) {
        if let Ok(mut state) = buzzer_state.lock() {
            for frame in data.chunks_mut(channels as usize) {
                let sample = if state.playing {
                    let waveform_sample = (state.phase * 2.0 * std::f32::consts::PI).sin();
                    state.phase += state.frequency / sample_rate;
                    
                    if state.phase >= 1.0 {
                        state.phase -= 1.0;
                    }
                    
                    let float_sample = waveform_sample * state.volume;
                    ((float_sample + 1.0) * 0.5 * u16::MAX as f32) as u16
                } else {
                    u16::MAX / 2 // Middle value for unsigned
                };
                
                for channel_sample in frame {
                    *channel_sample = sample;
                }
            }
        } else {
            data.fill(u16::MAX / 2);
        }
    }
}

/// Lists available audio devices.
pub fn list_audio_devices() -> Result<Vec<String>, AudioError> {
    let host = cpal::default_host();
    let devices = host.output_devices()
        .map_err(|e| AudioError::StreamError(e.to_string()))?;
    
    let mut device_names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }
    
    Ok(device_names)
}

/// Gets the default audio device information.
pub fn get_default_device_info() -> Result<String, AudioError> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or(AudioError::DeviceNotAvailable)?;
    
    device.name()
        .map_err(|e| AudioError::StreamError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_config_creation() {
        let config = StreamConfig::new();
        
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 1);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.sample_format, StreamSampleFormat::F32);
    }
    
    #[test]
    fn test_stream_config_builder() {
        let config = StreamConfig::new()
            .with_sample_rate(48000)
            .with_channels(2)
            .with_buffer_size(512)
            .with_sample_format(StreamSampleFormat::I16);
        
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.sample_format, StreamSampleFormat::I16);
    }
    
    #[test]
    fn test_stream_config_presets() {
        let low_latency = StreamConfig::low_latency();
        assert_eq!(low_latency.buffer_size, 256);
        assert_eq!(low_latency.sample_rate, 48000);
        
        let high_quality = StreamConfig::high_quality();
        assert_eq!(high_quality.buffer_size, 2048);
        assert_eq!(high_quality.sample_rate, 96000);
        assert_eq!(high_quality.channels, 2);
    }
    
    #[test]
    fn test_sample_format_conversion() {
        assert_eq!(SampleFormat::from(StreamSampleFormat::F32), SampleFormat::F32);
        assert_eq!(SampleFormat::from(StreamSampleFormat::I16), SampleFormat::I16);
        assert_eq!(SampleFormat::from(StreamSampleFormat::U16), SampleFormat::U16);
    }
    
    #[test]
    fn test_audio_device_listing() {
        // This test may fail in headless environments
        match list_audio_devices() {
            Ok(_devices) => {
                #[cfg(feature = "debug-print")]
                println!("Available audio devices: {:?}", _devices);
                // Just check that it doesn't panic
            }
            Err(_) => {
                // It's okay if this fails in CI/headless environments
                #[cfg(feature = "debug-print")]
                println!("No audio devices available (headless environment)");
            }
        }
    }
    
    #[test]
    fn test_default_device_info() {
        // This test may fail in headless environments
        match get_default_device_info() {
            Ok(device_name) => {
                #[cfg(feature = "debug-print")]
                println!("Default audio device: {}", device_name);
                assert!(!device_name.is_empty());
            }
            Err(_) => {
                // It's okay if this fails in CI/headless environments
                #[cfg(feature = "debug-print")]
                println!("No default audio device (headless environment)");
            }
        }
    }
    
    // Note: AudioStream creation tests require an actual audio device,
    // so they would fail in headless CI environments.
    // For integration testing, these would need to be run manually
    // or in environments with audio hardware.
}