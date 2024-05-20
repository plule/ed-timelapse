use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Context, Result};
use ed_journals::{journal_event_content::screenshot_event::ScreenshotEvent, JournalEventContent};

pub struct Exit;

fn get_journal_dir() -> Result<std::path::PathBuf> {
    let home = directories::UserDirs::new()
        .context("Failed to find home directory")?
        .home_dir()
        .to_owned();
    let journal_dir = home
        .join("Saved Games")
        .join("Frontier Developments")
        .join("Elite Dangerous");
    Ok(journal_dir)
}

pub fn watch_screenshots() -> Result<(Receiver<ScreenshotEvent>, Sender<Exit>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (exit_tx, exit_rx) = std::sync::mpsc::channel();
    let journals = ed_journals::JournalDir::new(get_journal_dir()?)?;
    let reader = journals
        .journal_logs_newest_first()?
        .get(0)
        .context("No journal files")?
        .create_live_reader()
        .context("Failed to read the journal file")?;

    std::thread::spawn(move || {
        for event in reader {
            // bug, won't stop if no event arrive in the journal
            if exit_rx.try_recv().is_ok() {
                return;
            }
            match event {
                Ok(event) => {
                    if let JournalEventContent::Screenshot(screenshot_event) = event.content {
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
