use crossbeam_channel::Sender;
use log::error;
use rdev::{listen as rdev_listen, Event, EventType, Key};
use std::collections::HashSet;

use crate::keys::{KeyCategory, KeyEvent};

fn map_key(key: Key) -> KeyCategory {
    match key {
        Key::Space => KeyCategory::Space,
        Key::Return => KeyCategory::Enter,
        Key::Backspace => KeyCategory::Backspace,
        Key::Escape => KeyCategory::Escape,
        Key::ShiftLeft
        | Key::ShiftRight
        | Key::ControlLeft
        | Key::ControlRight
        | Key::Alt
        | Key::AltGr
        | Key::MetaLeft
        | Key::MetaRight
        | Key::CapsLock
        | Key::Tab => KeyCategory::Modifier,
        _ => KeyCategory::Generic,
    }
}

pub fn listen(tx: Sender<KeyEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let mut pressed_keys = HashSet::new();

    let callback = move |event: Event| {
        let (category, is_press) = match event.event_type {
            EventType::KeyPress(key) => {
                if !pressed_keys.insert(key) {
                    // Suppress typematic auto-repeat when key is held down
                    return;
                }
                (map_key(key), true)
            }
            EventType::KeyRelease(key) => {
                pressed_keys.remove(&key);
                (map_key(key), false)
            }
            _ => return,
        };
        let _ = tx.send(KeyEvent { category, is_press });
    };

    if let Err(error) = rdev_listen(callback) {
        error!("rdev listen failed: {:?}", error);
        return Err(format!("rdev listen error: {:?}", error).into());
    }

    Ok(())
}
