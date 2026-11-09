mod audio;
mod config;
mod input;
mod keys;

use clap::Parser;
use crossbeam_channel::unbounded;
use log::{error, info};
use std::path::PathBuf;
use std::thread;

/// Clack - Mechanical keyboard sound emulator
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(name = "clack", version, about = "Mechanical keyboard sound emulator", long_about = None)]
pub struct Cli {
    /// Custom sound pack directory
    #[arg(short, long, value_name = "PATH")]
    pub soundpack: Option<PathBuf>,

    /// Volume level (0.0 to 1.0)
    #[arg(short, long, default_value_t = 0.8, value_parser = validate_volume)]
    pub volume: f32,
}

fn validate_volume(val: &str) -> Result<f32, String> {
    let v: f32 = val
        .parse()
        .map_err(|_| format!("'{val}' is not a valid number"))?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("Volume must be between 0.0 and 1.0, got {v}"))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let config = config::Config::from(cli);

    info!("Starting Clack mechanical keyboard sound emulator");
    info!("Volume: {:.0}%", config.volume * 100.0);

    // Load sound pack
    let sounds = match &config.soundpack {
        Some(path) => {
            info!("Loading custom sound pack from: {}", path.display());
            audio::SoundPack::load_from_directory(path)?
        }
        None => {
            info!("Loading embedded default sound pack");
            audio::SoundPack::load_embedded()?
        }
    };

    info!("Loaded sound pack with {} sound(s)", sounds.total_sounds());

    // Create crossbeam channel
    let (tx, rx) = unbounded();

    // Create player and spawn audio thread
    let mut player = audio::player::Player::new(config.volume, sounds)?;
    let audio_thread = thread::spawn(move || {
        player.run(rx);
    });

    info!("Listening for keyboard input... (Press Ctrl+C to exit)");

    // Call input listener on main thread (blocks until Ctrl+C or error)
    if let Err(e) = input::listen(tx) {
        error!("Input error: {}", e);
        return Err(e);
    }

    // Wait for audio thread to complete
    let _ = audio_thread.join();
    info!("Clack shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_volume_valid() {
        assert_eq!(validate_volume("0.0"), Ok(0.0));
        assert_eq!(validate_volume("0.5"), Ok(0.5));
        assert_eq!(validate_volume("0.8"), Ok(0.8));
        assert_eq!(validate_volume("1.0"), Ok(1.0));
    }

    #[test]
    fn test_validate_volume_out_of_range() {
        assert!(validate_volume("-0.1").is_err());
        assert!(validate_volume("1.01").is_err());
        assert!(validate_volume("2.0").is_err());
        assert!(validate_volume("-10.0").is_err());
    }

    #[test]
    fn test_validate_volume_invalid_string() {
        assert!(validate_volume("abc").is_err());
        assert!(validate_volume("").is_err());
        assert!(validate_volume("0.5foo").is_err());
    }

    #[test]
    fn test_cli_parsing_defaults() {
        let cli = Cli::try_parse_from(["clack"]).unwrap();
        assert_eq!(cli.soundpack, None);
        assert!((cli.volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cli_parsing_custom_args() {
        let cli = Cli::try_parse_from(["clack", "--volume", "0.3", "--soundpack", "/tmp/sounds"])
            .unwrap();
        assert_eq!(cli.soundpack, Some(PathBuf::from("/tmp/sounds")));
        assert!((cli.volume - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cli_parsing_invalid_volume() {
        let result = Cli::try_parse_from(["clack", "--volume", "1.5"]);
        assert!(result.is_err());
    }
}
