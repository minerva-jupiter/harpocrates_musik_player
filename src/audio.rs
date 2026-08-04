pub struct AudioEngine {
    player: rodio::Player,
    stream_handle: rodio::MixerDeviceSink,
}
pub trait AudioPlayer {
    fn new() -> Result<AudioEngine, Box<dyn std::error::Error>>;
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
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
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
