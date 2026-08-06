use crate::setting::AudioSettings;
use rodio::Device;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::stream::{DeviceSinkBuilder, MixerDeviceSink};

pub struct AudioEngine {
    player: rodio::Player,
    stream_handle: rodio::MixerDeviceSink,
}
pub trait AudioPlayer {
    fn new(audio_settings: &AudioSettings) -> Result<AudioEngine, Box<dyn std::error::Error>>;
    fn append_a_track(&mut self, file: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn is_paused(&self) -> bool;
    fn duration(&self) -> std::time::Duration;
    fn try_seek(&mut self, position: std::time::Duration)
    -> Result<(), Box<dyn std::error::Error>>;
    fn is_empty(&self) -> bool;
    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);
    fn append_tracks(&mut self, files: &[&str]) -> Result<(), Box<dyn std::error::Error>>;
}

impl AudioPlayer for AudioEngine {
    fn new(audio_settings: &AudioSettings) -> Result<Self, Box<dyn std::error::Error>> {
        let stream_handle = DeviceManager::open_sink(&audio_settings.output_device())?;
        let player = rodio::Player::connect_new(stream_handle.mixer());
        Ok(Self {
            player,
            stream_handle,
        })
    }
    fn append_a_track(&mut self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(file)?;
        self.player.append(rodio::Decoder::try_from(file)?);
        Ok(())
    }
    fn play(&mut self) {
        self.player.play();
    }
    fn pause(&mut self) {
        self.player.pause();
    }
    fn stop(&mut self) {
        self.player.stop();
    }
    fn is_paused(&self) -> bool {
        self.player.is_paused()
    }
    fn try_seek(
        &mut self,
        position: std::time::Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.player.try_seek(position)?;
        Ok(())
    }
    fn duration(&self) -> std::time::Duration {
        self.player.get_pos()
    }
    fn is_empty(&self) -> bool {
        self.player.empty()
    }
    fn volume(&self) -> f32 {
        self.player.volume()
    }
    fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
    }
    fn append_tracks(&mut self, files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        for file in files {
            self.append_a_track(file)?;
        }
        Ok(())
    }
}

pub struct DeviceManager;

impl DeviceManager {
    pub fn list_device_names() -> Vec<String> {
        let host = rodio::cpal::default_host();
        let Ok(devices) = host.output_devices() else {
            return Vec::new();
        };

        devices.filter_map(|d| d.name().ok()).collect()
    }

    pub fn find_device_by_name(name: &str) -> Option<Device> {
        if name.trim().is_empty() {
            return None;
        }

        let host = rodio::cpal::default_host();
        let Ok(mut devices) = host.output_devices() else {
            return None;
        };

        devices.find(|d| d.name().map(|n| n == name).unwrap_or(false))
    }

    pub fn open_sink(device_name: &str) -> Result<MixerDeviceSink, Box<dyn std::error::Error>> {
        if let Some(device) = Self::find_device_by_name(device_name) {
            if let Ok(builder) = DeviceSinkBuilder::from_device(device) {
                if let Ok(sink) = builder.open_stream() {
                    return Ok(sink);
                }
            }
        }

        let sink = DeviceSinkBuilder::open_default_sink()?;
        Ok(sink)
    }
}
