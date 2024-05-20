use std::{mem::size_of, thread, time::Duration};

use anyhow::{bail, Context, Result};
use windows::{
    core::{w, PCWSTR},
    Win32::UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY,
        },
        WindowsAndMessaging::{
            BringWindowToTop, FindWindowW, GetForegroundWindow, SetForegroundWindow,
        },
    },
};

fn keyscan_input(wscan: u16, dwflag: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: wscan,
                dwFlags: KEYBD_EVENT_FLAGS(dwflag),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

const KEYSCAN_F10: u16 = 0x44;
const KEYSCAN_LALT: u16 = 0x38;

pub fn request_screenshot(high_res: bool) -> Result<()> {
    unsafe {
        let active_window = GetForegroundWindow();
        let ed_window = FindWindowW(PCWSTR::null(), w!("Elite - Dangerous (CLIENT)"));
        if ed_window.0 == 0 {
            bail!("Elite Dangerous does not appear to be running.");
        }

        if !SetForegroundWindow(ed_window).as_bool() {
            bail!("Failed to bring Elite Dangerous to the foreground.");
        }

        BringWindowToTop(ed_window).context("Failed to bring Elite Dangerous to the top.")?;

        let keys = if high_res {
            vec![KEYSCAN_LALT, KEYSCAN_F10]
        } else {
            vec![KEYSCAN_F10]
        };

        thread::sleep(Duration::from_millis(60));
        let sent = SendInput(
            &keys
                .iter()
                .map(|k| keyscan_input(*k, 0))
                .collect::<Vec<_>>(),
            size_of::<INPUT>() as i32,
        );
        if sent != keys.len() as u32 {
            bail!("Failed to send down keypresses.");
        }
        thread::sleep(Duration::from_millis(60));
        let sent = SendInput(
            &keys
                .iter()
                .rev()
                .map(|k| keyscan_input(*k, KEYEVENTF_KEYUP.0))
                .collect::<Vec<_>>(),
            size_of::<INPUT>() as i32,
        );
        if sent != keys.len() as u32 {
            bail!("Failed to send up keypresses.");
        }
        //thread::sleep(Duration::from_millis(60));
        let _ = SetForegroundWindow(active_window);
    }
    Ok(())
}
