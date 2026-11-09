#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCategory {
    Generic,
    Space,
    Enter,
    Backspace,
    Escape,
    Modifier,
}

impl KeyCategory {
    /// Parse a key category from a sound filename prefix (e.g., "space_1.wav" -> "space" -> KeyCategory::Space)
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix.to_ascii_lowercase().as_str() {
            "generic" => Some(Self::Generic),
            "space" => Some(Self::Space),
            "enter" => Some(Self::Enter),
            "backspace" => Some(Self::Backspace),
            "escape" | "esc" => Some(Self::Escape),
            "modifier" | "mod" => Some(Self::Modifier),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub category: KeyCategory,
    pub is_press: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_prefix_valid_categories() {
        assert_eq!(
            KeyCategory::from_prefix("generic"),
            Some(KeyCategory::Generic)
        );
        assert_eq!(KeyCategory::from_prefix("space"), Some(KeyCategory::Space));
        assert_eq!(KeyCategory::from_prefix("enter"), Some(KeyCategory::Enter));
        assert_eq!(
            KeyCategory::from_prefix("backspace"),
            Some(KeyCategory::Backspace)
        );
        assert_eq!(
            KeyCategory::from_prefix("escape"),
            Some(KeyCategory::Escape)
        );
        assert_eq!(KeyCategory::from_prefix("esc"), Some(KeyCategory::Escape));
        assert_eq!(
            KeyCategory::from_prefix("modifier"),
            Some(KeyCategory::Modifier)
        );
        assert_eq!(KeyCategory::from_prefix("mod"), Some(KeyCategory::Modifier));
    }

    #[test]
    fn test_from_prefix_case_insensitivity() {
        assert_eq!(
            KeyCategory::from_prefix("GENERIC"),
            Some(KeyCategory::Generic)
        );
        assert_eq!(KeyCategory::from_prefix("Space"), Some(KeyCategory::Space));
        assert_eq!(KeyCategory::from_prefix("ENTER"), Some(KeyCategory::Enter));
        assert_eq!(
            KeyCategory::from_prefix("BackSpace"),
            Some(KeyCategory::Backspace)
        );
        assert_eq!(KeyCategory::from_prefix("ESC"), Some(KeyCategory::Escape));
        assert_eq!(KeyCategory::from_prefix("Mod"), Some(KeyCategory::Modifier));
    }

    #[test]
    fn test_from_prefix_invalid() {
        assert_eq!(KeyCategory::from_prefix(""), None);
        assert_eq!(KeyCategory::from_prefix("unknown"), None);
        assert_eq!(KeyCategory::from_prefix("key"), None);
        assert_eq!(KeyCategory::from_prefix("123"), None);
    }
}
