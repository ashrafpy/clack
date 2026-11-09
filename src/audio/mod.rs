pub mod player;

use include_dir::{include_dir, Dir};
use kira::sound::static_sound::StaticSoundData;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use crate::keys::KeyCategory;

static DEFAULT_SOUNDS: Dir = include_dir!("assets/sounds/default");

#[derive(Clone, Default)]
pub struct SoundPack {
    sounds: HashMap<KeyCategory, Vec<StaticSoundData>>,
}

impl SoundPack {
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
        }
    }

    pub fn add_sound(&mut self, category: KeyCategory, sound: StaticSoundData) {
        self.sounds.entry(category).or_default().push(sound);
    }

    pub fn get(&self, category: KeyCategory) -> &[StaticSoundData] {
        self.sounds
            .get(&category)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn total_sounds(&self) -> usize {
        self.sounds.values().map(|v| v.len()).sum()
    }

    /// Load the default sound pack embedded at compile time
    pub fn load_embedded() -> Result<Self, Box<dyn std::error::Error>> {
        let mut pack = Self::new();
        let mut count = 0;

        for file in DEFAULT_SOUNDS.files() {
            let path_str = file.path().to_string_lossy();
            if !path_str.ends_with(".wav") {
                continue;
            }

            if let Some(category) = parse_category_from_path(file.path()) {
                let cursor = Cursor::new(file.contents());
                match StaticSoundData::from_cursor(cursor) {
                    Ok(sound) => {
                        debug!("Loaded embedded sound: {} ({:?})", path_str, category);
                        pack.add_sound(category, sound);
                        count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load embedded sound {}: {}", path_str, e);
                    }
                }
            } else {
                warn!(
                    "Could not determine key category for embedded sound: {}",
                    path_str
                );
            }
        }

        if count == 0 {
            warn!("No embedded sound files could be loaded!");
        } else {
            info!("Loaded {} embedded sound effects", count);
        }

        Ok(pack)
    }

    /// Load custom sound pack from a directory on disk
    pub fn load_from_directory<P: AsRef<Path>>(
        dir_path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = dir_path.as_ref();
        if !path.is_dir() {
            return Err(format!("Soundpack path '{}' is not a directory", path.display()).into());
        }

        let mut pack = Self::new();
        let mut count = 0;

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }

            let is_wav = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("wav"))
                .unwrap_or(false);

            if !is_wav {
                continue;
            }

            if let Some(category) = parse_category_from_path(&entry_path) {
                match StaticSoundData::from_file(&entry_path) {
                    Ok(sound) => {
                        debug!(
                            "Loaded sound file: {} ({:?})",
                            entry_path.display(),
                            category
                        );
                        pack.add_sound(category, sound);
                        count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load sound file {}: {}", entry_path.display(), e);
                    }
                }
            } else {
                warn!(
                    "Could not determine key category for file: {}",
                    entry_path.display()
                );
            }
        }

        if count == 0 {
            return Err(format!("No valid .wav sounds found in '{}'", path.display()).into());
        }

        info!("Loaded {} sound effects from '{}'", count, path.display());
        Ok(pack)
    }
}

fn parse_category_from_path(path: &Path) -> Option<KeyCategory> {
    let file_stem = path.file_stem()?.to_str()?;
    // Filename formats like "generic_1", "space_2", or "enter"
    let prefix = file_stem.split('_').next().unwrap_or(file_stem);
    KeyCategory::from_prefix(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category_from_path() {
        assert_eq!(
            parse_category_from_path(Path::new("generic_1.wav")),
            Some(KeyCategory::Generic)
        );
        assert_eq!(
            parse_category_from_path(Path::new("space_2.wav")),
            Some(KeyCategory::Space)
        );
        assert_eq!(
            parse_category_from_path(Path::new("/path/to/enter_1.wav")),
            Some(KeyCategory::Enter)
        );
        assert_eq!(
            parse_category_from_path(Path::new("backspace_1.wav")),
            Some(KeyCategory::Backspace)
        );
        assert_eq!(
            parse_category_from_path(Path::new("escape_1.wav")),
            Some(KeyCategory::Escape)
        );
        assert_eq!(
            parse_category_from_path(Path::new("esc_1.wav")),
            Some(KeyCategory::Escape)
        );
        assert_eq!(
            parse_category_from_path(Path::new("modifier_1.wav")),
            Some(KeyCategory::Modifier)
        );
        assert_eq!(
            parse_category_from_path(Path::new("mod_2.wav")),
            Some(KeyCategory::Modifier)
        );
        assert_eq!(parse_category_from_path(Path::new("nonexistent.wav")), None);
    }

    #[test]
    fn test_load_embedded_soundpack() {
        let pack =
            SoundPack::load_embedded().expect("Default embedded sound pack must load successfully");
        assert_eq!(
            pack.total_sounds(),
            12,
            "Should load exactly 12 embedded WAV sounds"
        );

        // Verify all 6 KeyCategory variants are populated
        let generic_sounds = pack.get(KeyCategory::Generic);
        assert_eq!(generic_sounds.len(), 4, "Generic sounds count");

        let space_sounds = pack.get(KeyCategory::Space);
        assert_eq!(space_sounds.len(), 2, "Space sounds count");

        let enter_sounds = pack.get(KeyCategory::Enter);
        assert_eq!(enter_sounds.len(), 2, "Enter sounds count");

        let backspace_sounds = pack.get(KeyCategory::Backspace);
        assert_eq!(backspace_sounds.len(), 1, "Backspace sounds count");

        let escape_sounds = pack.get(KeyCategory::Escape);
        assert_eq!(escape_sounds.len(), 1, "Escape sounds count");

        let modifier_sounds = pack.get(KeyCategory::Modifier);
        assert_eq!(modifier_sounds.len(), 2, "Modifier sounds count");
    }
}
