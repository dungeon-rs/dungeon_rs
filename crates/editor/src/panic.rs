//! Registers a custom `panic!` handler that alerts the user of unrecoverable errors.

use bevy::prelude::error;
use rfd::{MessageButtons, MessageDialog};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use sysinfo::System;

/// Registers a new `panic!` handler that alerts the user of unrecoverable errors.
pub fn register_panic_handler() {
    let default_hook = std::panic::take_hook();
    #[cfg(not(test))]
    std::panic::set_hook(Box::new(move |info| {
        let path = std::env::current_dir()
            .unwrap_or(PathBuf::from("."))
            .join("crash_report.txt");
        let message = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            String::from(*message)
        } else {
            String::from("An unrecoverable error has occurred.")
        };
        let location = if let Some(location) = info.location() {
            location.to_string()
        } else {
            String::from("Unknown location")
        };

        error!("An unrecoverable error has occurred: {:?}", info);
        MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Unrecoverable Error")
            .set_buttons(MessageButtons::Ok)
            .set_description(format!(
                "An unrecoverable error has occurred, the editor will shut down.
The error was: {message}

Error occurred at: {location}

A crash file will be generated at {path:?}"
            ))
            .show();

        let system = System::new_all();
        let os_version = System::long_os_version().unwrap_or(String::from("Unknown"));
        let mut dump_file = File::create(path).expect("Failed to create dump file");
        writeln!(dump_file, "--- DungeonRS Crash Report ---").unwrap();
        writeln!(
            dump_file,
            "Please provide the contents of this file when creating a bug report."
        )
        .unwrap();
        writeln!(dump_file).unwrap();
        writeln!(dump_file).unwrap();
        writeln!(dump_file, "Operating System: {}", os_version).unwrap();
        writeln!(
            dump_file,
            "Memory: {}/{}",
            system.used_memory(),
            system.total_memory()
        )
        .unwrap();

        writeln!(
            dump_file,
            "CPU: {} Cores {}",
            system.cpus().len(),
            system.cpus()[0].brand()
        )
        .unwrap();
        for cpu in system.cpus() {
            writeln!(
                dump_file,
                "{}: {}Hz ({}) {}%",
                cpu.name(),
                cpu.frequency(),
                cpu.brand(),
                cpu.cpu_usage()
            )
            .unwrap();
        }

        writeln!(dump_file).unwrap();
        writeln!(dump_file, "---").unwrap();
        writeln!(dump_file, "Error: {}", message).unwrap();
        writeln!(dump_file, "Location: {}", location).unwrap();
        writeln!(dump_file, "---").unwrap();
        writeln!(dump_file, "Raw: {:?}", info).unwrap();

        dump_file.flush();
        default_hook(info);
    }));
}
