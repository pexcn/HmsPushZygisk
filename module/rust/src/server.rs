use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    sync::Once,
};

use android_logger::Config;
use log::{debug, error, LevelFilter};

const CONFIG_PATH: &str = "/data/misc/hmspush/app.conf";

/// Companion handler: reads package_name and process_name from the zygote side,
/// checks the config file, and writes back 1 byte: 1 = should hook, 0 = skip.
pub fn companion_handler(stream: &mut UnixStream) {
    static LOG_INIT: Once = Once::new();
    LOG_INIT.call_once(|| {
        android_logger::init_once(
            Config::default()
                .with_max_level(LevelFilter::Debug)
                .with_tag("HmsPushZygiskServer"),
        );
    });

    match handle_query(stream) {
        Ok(should_hook) => {
            debug!("Query done, should_hook = {}", should_hook);
        }
        Err(e) => {
            error!("companion_handler error: {}", e);
            // best-effort: tell the client not to hook
            let _ = stream.write_all(&[0u8]);
        }
    }
}

fn handle_query(stream: &mut UnixStream) -> io::Result<bool> {
    // Read two newline-terminated strings sent by the zygote side.
    let mut reader = BufReader::new(&mut *stream);

    let mut package_name = String::new();
    reader.read_line(&mut package_name)?;
    let package_name = package_name.trim_end();

    let mut process_name = String::new();
    reader.read_line(&mut process_name)?;
    let process_name = process_name.trim_end();

    debug!(
        "companion query: package = [{}], process = [{}]",
        package_name, process_name
    );

    let should_hook = check_config(package_name, process_name)?;

    // Reply with a single byte.
    stream.write_all(&[should_hook as u8])?;
    stream.flush()?;

    Ok(should_hook)
}

/// Parse the config file and decide whether the given (package, process) should be hooked.
fn check_config(package_name: &str, process_name: &str) -> io::Result<bool> {
    let content = match fs::read_to_string(CONFIG_PATH) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            debug!("Config file not found, skip hook");
            return Ok(false);
        }
        Err(e) => return Err(e),
    };

    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split_once('|') {
            Some((pkg, proc)) if pkg == package_name => {
                // A process-specific entry: match only when proc is empty (meaning all
                // processes of this package) or equals the actual process name.
                if proc.is_empty() || proc == process_name {
                    return Ok(true);
                }
            }
            None if line == package_name => {
                // A package-only entry means all processes of this package should be hooked.
                return Ok(true);
            }
            _ => {}
        }
    }

    Ok(false)
}
