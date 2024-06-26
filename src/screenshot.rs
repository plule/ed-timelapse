use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use ed_journals::logs::content::log_event_content::screenshot_event::ScreenshotEvent;

use self::watch::Exit;

mod request;
mod watch;

#[derive(Debug)]
pub struct Screenshot {
    pub path: PathBuf,
    pub location: String,
}

pub struct Watcher {
    rx: Receiver<ScreenshotEvent>,
    exit_tx: Sender<watch::Exit>,
}

impl Watcher {
    pub fn try_new() -> Result<Self> {
        let (rx, exit_tx) = watch::watch_screenshots()?;
        Ok(Self { rx, exit_tx })
    }

    pub fn take_screenshot(&mut self, high_res: bool) -> Result<Screenshot> {
        // Empty the screenshot channel
        while self.rx.recv_timeout(Duration::from_millis(100)).is_ok() {}

        // Request a screenshot
        request::request_screenshot(high_res)?;

        // Wait for the screenshot
        let screenshot = self.rx.recv_timeout(Duration::from_secs(10))?;

        screenshot.try_into()
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        if let Err(e) = self.exit_tx.send(Exit) {
            log::error!("Failed to send exit signal to screenshot watcher: {}", e);
        }
    }
}

impl TryFrom<ScreenshotEvent> for Screenshot {
    type Error = anyhow::Error;

    fn try_from(value: ScreenshotEvent) -> Result<Self, Self::Error> {
        let picture_dir = directories::UserDirs::new()
            .context("Failed to find home directory")?
            .picture_dir()
            .context("Failed to find picture directory")?
            .to_owned();
        let screenshot_dir = picture_dir
            .join("Frontier Developments")
            .join("Elite Dangerous");

        // weird ED_Pictures prefix in the file name
        let filename = value
            .filename
            .split('\\')
            .last()
            .context("Failed to split")?;
        let path = screenshot_dir.join(filename);
        if !path.is_file() {
            bail!("Screenshot file does not exist");
        }
        let location = value
            .body
            .or(value.system)
            .unwrap_or_else(|| "Unknown location".to_string());
        Ok(Self { path, location })
    }
}
