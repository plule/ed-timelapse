use std::{
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use log::info;

use crate::screenshot::{Screenshot, Watcher};

pub struct Exit;

#[derive(Debug, Clone)]
pub enum Status {
    Capturing,
    Waiting(Instant),
}

#[derive(Debug)]
pub struct TimelapseControl {
    pub exit_tx: Sender<Exit>,
    status_rx: Receiver<Status>,
    pub status: Status,
}

impl TimelapseControl {
    pub fn start(
        folder: PathBuf,
        interval: Duration,
        high_res: bool,
        organize: bool,
        remove_original: bool,
    ) -> Result<Self> {
        let mut screenshot = crate::screenshot::Watcher::try_new()?;
        let (exit_tx, exit_rx) = std::sync::mpsc::channel();
        let (status_tx, status_rx) = std::sync::mpsc::channel();
        let start = Instant::now();
        let mut index = 0;
        thread::spawn(move || loop {
            if exit_rx.try_recv().is_ok() {
                info!("Stopping the timelapse");
                return;
            }
            status_tx.send(Status::Capturing).unwrap();
            match take_screenshot(
                &mut screenshot,
                high_res,
                organize,
                remove_original,
                &folder,
            ) {
                Ok(s) => {
                    log::info!("Screenshot taken: {}", s.display());
                }
                Err(e) => {
                    log::error!("Failed to take screenshot: {}", e);
                }
            }
            index += 1;
            let mut next = start + index as u32 * interval;
            while Instant::now() > next {
                log::warn!("Missed a screenshot");
                index += 1;
                next = start + index as u32 * interval;
            }
            let _ = status_tx.send(Status::Waiting(next));
            thread::sleep(next - Instant::now());
        });
        Ok(Self {
            exit_tx,
            status_rx,
            status: Status::Capturing,
        })
    }

    pub fn update_status(&mut self) {
        if let Some(status) = self.status_rx.try_iter().last() {
            self.status = status;
        }
    }

    pub fn stop(&self) {
        if let Err(e) = self.exit_tx.send(Exit) {
            log::error!("Failed to send exit signal to timelapse: {}", e);
        }
    }
}

pub fn take_screenshot(
    watcher: &mut Watcher,
    high_res: bool,
    organize: bool,
    remove_original: bool,
    folder: &Path,
) -> Result<PathBuf> {
    watcher.take_screenshot(high_res).and_then(|s| {
        if organize {
            store_screenshot(s, remove_original, folder)
        } else {
            Ok(s.path)
        }
    })
}

fn store_screenshot(
    screenshot: Screenshot,
    remove_original: bool,
    folder: &Path,
) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let folder = folder.join(format!(
        "{} {}",
        now.format("%Y-%m-%d"),
        screenshot.location,
    ));
    let image = image::io::Reader::open(&screenshot.path)?.decode()?;
    std::fs::create_dir_all(&folder)?;
    let filename = now.format("%H-%M-%S.jpg").to_string();
    let destination = folder.join(filename);
    image.save(&destination)?;

    if remove_original {
        info!("Removing original screenshot: {:?}", screenshot.path);
        std::fs::remove_file(screenshot.path)?;
    }

    Ok(destination)
}
