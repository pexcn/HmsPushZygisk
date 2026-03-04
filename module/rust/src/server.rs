use std::{
    fs::File,
    io::{self, Write},
    os::unix::net::UnixStream,
};

use log::{debug, error};

const CONFIG_PATH: &str = "/data/misc/hmspush/app.conf";

/// Companion handler: reads CONFIG_PATH and streams it to the zygote-side module.
/// Protocol: write 8 bytes (i64 LE) file size, then file content.
pub fn companion_handler(stream: &mut UnixStream) {
    match send_file(stream, CONFIG_PATH) {
        Ok(size) => debug!("Sent module payload: {} bytes", size),
        Err(e) => error!("Failed to send config: {}", e),
    }
}

fn send_file(stream: &mut UnixStream, path: &str) -> io::Result<u64> {
    let mut file = File::open(path).map_err(|e| {
        debug!("Failed to open file {}: {}", path, e);
        e
    })?;

    let size = file.metadata()?.len();

    // Send size header (8 bytes, little-endian i64)
    stream.write_all(&(size as i64).to_le_bytes())?;

    // Stream file content
    let copied = io::copy(&mut file, stream)?;
    debug!("Copied {} bytes of config", copied);

    Ok(copied)
}
