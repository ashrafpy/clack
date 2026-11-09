#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod other;

use crate::keys::KeyEvent;
use crossbeam_channel::Sender;

pub fn listen(tx: Sender<KeyEvent>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    linux::listen(tx)?;
    #[cfg(not(target_os = "linux"))]
    other::listen(tx)?;
    Ok(())
}
