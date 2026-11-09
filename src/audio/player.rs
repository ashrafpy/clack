use crossbeam_channel::Receiver;
use kira::{
    sound::static_sound::StaticSoundSettings, AudioManager, AudioManagerSettings, DefaultBackend,
};
use log::debug;
use rand::Rng;

use crate::audio::SoundPack;
use crate::keys::{KeyCategory, KeyEvent};

pub struct Player {
    manager: AudioManager,
    sounds: SoundPack,
    volume: f64,
}

impl Player {
    pub fn new(volume: f32, sounds: SoundPack) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        Ok(Self {
            manager,
            sounds,
            volume: volume.clamp(0.0, 1.0) as f64,
        })
    }

    pub fn run(&mut self, rx: Receiver<KeyEvent>) {
        let mut rng = rand::rng();
        for event in rx {
            // Only play on key press (no key-up sounds for v1)
            if !event.is_press {
                continue;
            }

            // Get sounds for this category, falling back to Generic if empty
            let mut sounds = self.sounds.get(event.category);
            if sounds.is_empty() {
                sounds = self.sounds.get(KeyCategory::Generic);
            }
            if sounds.is_empty() {
                continue;
            }

            // Pick random sound from variants
            let idx = rng.random_range(0..sounds.len());
            let sound = &sounds[idx];

            // Random pitch jitter ±3%
            let pitch: f64 = 1.0 + rng.random_range(-0.03..0.03);
            // Random volume jitter ±5%
            let vol_mult: f64 = 1.0 + rng.random_range(-0.05..0.05);
            let linear_vol: f64 = (self.volume * vol_mult).clamp(0.0, 1.0);

            // Convert linear volume to decibels (0.0 -> -60dB silence, 1.0 -> 0dB unity)
            let db: f32 = if linear_vol <= 0.0001 {
                -60.0
            } else {
                20.0 * (linear_vol as f32).log10()
            };

            // Play with settings
            let settings = StaticSoundSettings::default()
                .playback_rate(pitch)
                .volume(db);

            let _ = self.manager.play(sound.with_settings(settings));
        }
        debug!("Audio player thread finished");
    }
}
