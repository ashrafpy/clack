use crossbeam_channel::Sender;
use evdev::{InputEventKind, Key};
use log::{debug, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::keys::{KeyCategory, KeyEvent};

fn map_key(key: Key) -> KeyCategory {
    match key {
        Key::KEY_SPACE => KeyCategory::Space,
        Key::KEY_ENTER => KeyCategory::Enter,
        Key::KEY_BACKSPACE => KeyCategory::Backspace,
        Key::KEY_ESC => KeyCategory::Escape,
        Key::KEY_LEFTSHIFT
        | Key::KEY_RIGHTSHIFT
        | Key::KEY_LEFTCTRL
        | Key::KEY_RIGHTCTRL
        | Key::KEY_LEFTALT
        | Key::KEY_RIGHTALT
        | Key::KEY_LEFTMETA
        | Key::KEY_RIGHTMETA
        | Key::KEY_CAPSLOCK
        | Key::KEY_TAB => KeyCategory::Modifier,
        _ => KeyCategory::Generic,
    }
}

pub fn listen(tx: Sender<KeyEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let mut keyboard_devices = Vec::new();

    for (path, device) in evdev::enumerate() {
        if let Some(keys) = device.supported_keys() {
            if keys.contains(Key::KEY_A) {
                let name = device.name().unwrap_or("Unknown keyboard").to_string();
                debug!("Discovered keyboard device: {} ({})", name, path.display());
                keyboard_devices.push((path, device, name));
            }
        }
    }

    if keyboard_devices.is_empty() {
        eprintln!(
            "\nError: No accessible keyboard devices found in /dev/input/.\n\
            Make sure your user has read permissions for /dev/input/event*.\n\
            You can add your user to the 'input' group with:\n\
                sudo usermod -aG input $USER\n\
            Then log out and back in for the change to take effect."
        );
        return Err("No accessible keyboard devices found in /dev/input/".into());
    }

    info!("Monitoring {} keyboard device(s)", keyboard_devices.len());

    let running = Arc::new(AtomicBool::new(true));

    // Register Ctrl+C handler to unpark main thread and stop listeners
    let r_ctrlc = running.clone();
    let main_thread = thread::current();
    let _ = ctrlc::set_handler(move || {
        info!("Received interrupt signal, stopping input listener...");
        r_ctrlc.store(false, Ordering::SeqCst);
        main_thread.unpark();
    });

    let mut thread_handles = Vec::new();

    for (path, mut device, name) in keyboard_devices {
        let dev_tx = tx.clone();
        let dev_running = running.clone();
        let handle = thread::spawn(move || {
            debug!("Starting event loop for {} ({})", name, path.display());
            while dev_running.load(Ordering::Relaxed) {
                match device.fetch_events() {
                    Ok(events) => {
                        for ev in events {
                            if let InputEventKind::Key(key) = ev.kind() {
                                // Filter out mouse buttons (BTN_*) — their codes
                                // start at 0x100 (256). Keyboard keys are 0-255.
                                if key.code() >= 0x100 {
                                    continue;
                                }

                                let is_press = match ev.value() {
                                    1 => true,
                                    0 => false,
                                    _ => continue, // Ignore auto-repeat (value == 2)
                                };
                                let category = map_key(key);
                                if dev_tx.send(KeyEvent { category, is_press }).is_err() {
                                    // Receiver dropped, stop thread
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Device {} error or disconnected: {}", name, e);
                        break;
                    }
                }
            }
        });
        thread_handles.push(handle);
    }

    // Drop our reference so only child threads hold Sender copies
    drop(tx);

    // Park main thread until signal received
    while running.load(Ordering::SeqCst) {
        thread::park();
    }

    info!("Input listener stopped");
    Ok(())
}
