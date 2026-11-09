use std::path::PathBuf;

/// Configuration options for Clack
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Optional path to a custom sound pack directory
    pub soundpack: Option<PathBuf>,
    /// Master volume (0.0 to 1.0)
    pub volume: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            soundpack: None,
            volume: 0.8,
        }
    }
}

impl From<crate::Cli> for Config {
    fn from(cli: crate::Cli) -> Self {
        Self {
            soundpack: cli.soundpack,
            volume: cli.volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.soundpack, None);
        assert!((config.volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_from_cli() {
        let cli = Cli {
            soundpack: Some(PathBuf::from("/path/to/sounds")),
            volume: 0.5,
        };
        let config = Config::from(cli);
        assert_eq!(config.soundpack, Some(PathBuf::from("/path/to/sounds")));
        assert!((config.volume - 0.5).abs() < f32::EPSILON);
    }
}
