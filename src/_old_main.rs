mod audio;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::audio::{AudioEngine, AudioPlayer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filepath = r"C:\Users\ryout\Music\ハチ\花束と水葬\01 - Persona Alice.flac";
    let mut audioengine: AudioEngine = audio::AudioEngine::new()?;
    audioengine.append_a_track(filepath)?;
    audioengine.set_volume(0.1);

    crossterm::terminal::enable_raw_mode()?;
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('p') => {
                        if audioengine.is_paused() {
                            audioengine.play();
                        } else {
                            audioengine.pause();
                        }
                    }
                    KeyCode::Char('s') => {
                        audioengine.stop();
                        break;
                    }
                    _ => {}
                }
            }
        }

        if audioengine.is_empty() {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
