use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};
use ed_journals::logs::content::{
    log_event_content::screenshot_event::ScreenshotEvent, LogEventContent,
};

pub struct Exit;

pub fn watch_screenshots() -> Result<(Receiver<ScreenshotEvent>, Sender<Exit>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (exit_tx, exit_rx) = std::sync::mpsc::channel();
    let journal_dir = ed_journals::journal::auto_detect_journal_path()
        .context("Failed to find the journal directory")?;
    let journals = ed_journals::logs::LogDir::new(journal_dir);
    let reader = journals
        .journal_logs_newest_first()?
        .first()
        .context("No journal files")?
        .create_live_blocking_reader()
        .context("Failed to read the journal file")?;

    std::thread::spawn(move || {
        for event in reader {
            // bug, won't stop if no event arrive in the journal
            if exit_rx.try_recv().is_ok() {
                return;
            }
            match event {
                Ok(event) => {
                    if let LogEventContent::Screenshot(screenshot_event) = event.content {
                        tx.send(screenshot_event).unwrap();
                    }
                }
                Err(err) => {
                    log::error!("Error reading journal event: {:?}", err);
                }
            }
        }
    });
    Ok((rx, exit_tx))
}
