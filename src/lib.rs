// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/process/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/process)
//!
//! This plugin provides APIs to access the current process. To spawn child processes, see the [`shell`](https://github.com/tauri-apps/tauri-plugin-shell) plugin.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use crate::commands::*;
use crate::state::SerialportState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod error;
mod state;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("serialplugin")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            available_ports,
            cancel_read,
            close,
            close_all,
            force_close,
            open,
            read,
            write,
            write_binary,
        ])
        .setup(|app, _| {
            let serialports = Arc::new(Mutex::new(HashMap::new()));

            // Wrap the HashMap in a SerialportState struct
            let serialport_state = SerialportState {
                serialports: serialports.clone(), // Cloning Arc to share ownership
            };
        
            // Manage the SerialportState in the Tauri application
            app.manage(serialport_state);
            Ok(())
        })
        .build()
}
