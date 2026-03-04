use std::os::unix::net::UnixStream;
use std::io::Read;

use jni::JNIEnv;
use log::{debug, error, info};
use zygisk_api::{
    ZygiskModule,
    api::{V4, ZygiskApi},
    raw::ZygiskRaw,
    register_module, register_companion,
};
// ZygiskOption is re-exported by the V4 transparent module via `pub use crate::raw::v4::transparent::*`
use zygisk_api::api::v4::ZygiskOption;

mod hook;
mod server;

#[derive(Default)]
struct HmsPushModule;

impl ZygiskModule for HmsPushModule {
    type Api = V4;

    fn pre_app_specialize<'a>(
        &self,
        mut api: ZygiskApi<'a, V4>,
        env: JNIEnv<'a>,
        args: &'a mut <V4 as ZygiskRaw<'_>>::AppSpecializeArgs,
    ) {
        // args.nice_name and args.app_data_dir are &JString<'a>
        let process_name = jstring_to_string(&env, args.nice_name);
        let app_data_dir = jstring_to_string(&env, args.app_data_dir);

        if process_name.is_empty() || app_data_dir.is_empty() {
            api.set_option(ZygiskOption::DlCloseModuleLibrary);
            return;
        }

        let package_name = parse_package_name(&app_data_dir);
        debug!(
            "preAppSpecialize, packageName = {}, process = {}",
            package_name, process_name
        );

        pre_specialize(api, env, &package_name, &process_name);
    }

    fn pre_server_specialize<'a>(
        &self,
        mut api: ZygiskApi<'a, V4>,
        _env: JNIEnv<'a>,
        _args: &'a mut <V4 as ZygiskRaw<'_>>::ServerSpecializeArgs,
    ) {
        api.set_option(ZygiskOption::DlCloseModuleLibrary);
    }
}

/// Convert a JString reference to a Rust String.
fn jstring_to_string(env: &JNIEnv<'_>, jstr: &jni::objects::JString<'_>) -> String {
    // SAFETY: env comes from Zygisk's trusted JNI entry which provides a valid env.
    let mut env_clone = unsafe { env.unsafe_clone() };
    match env_clone.get_string(jstr) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    }
}

/// Extract the last path component (package name) from an app_data_dir path.
/// Handles: /data/user/<uid>/<pkg>, /data/data/<pkg>, /mnt/expand/.../<pkg>
fn parse_package_name(app_data_dir: &str) -> String {
    app_data_dir
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("")
        .to_string()
}

fn pre_specialize(
    mut api: ZygiskApi<'_, V4>,
    env: JNIEnv<'_>,
    package_name: &str,
    process: &str,
) {
    let process_list = request_remote_config(&mut api, package_name);

    if !process_list.is_empty() {
        let should_hook = process_list
            .iter()
            .any(|item| item.is_empty() || item == process);

        if should_hook {
            info!("hook package = [{}], process = [{}]", package_name, process);
            hook::do_hook(&mut api, env);
            return;
        }
    }

    api.set_option(ZygiskOption::DlCloseModuleLibrary);
}

fn request_remote_config(
    api: &mut ZygiskApi<'_, V4>,
    package_name: &str,
) -> Vec<String> {
    debug!("requestRemoteConfig for {}", package_name);

    let result = api.with_companion(|stream| {
        receive_and_parse_config(stream, package_name)
    });

    match result {
        Ok(configs) => {
            debug!("config size: {}", configs.len());
            configs
        }
        Err(e) => {
            error!("Failed to connect to companion: {:?}", e);
            Vec::new()
        }
    }
}

fn receive_and_parse_config(stream: &mut UnixStream, package_name: &str) -> Vec<String> {
    let mut size_buf = [0u8; 8];
    match stream.read_exact(&mut size_buf) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            debug!("receive empty config");
            return Vec::new();
        }
        Err(e) => {
            error!("Failed to read size: {}", e);
            return Vec::new();
        }
    }

    let size = i64::from_le_bytes(size_buf);
    if size <= 0 {
        return Vec::new();
    }

    let mut content = vec![0u8; size as usize];
    if let Err(e) = stream.read_exact(&mut content) {
        error!("Failed to read config data: {}", e);
        return Vec::new();
    }

    parse_config(&content, package_name)
}

fn parse_config(content: &[u8], package_name: &str) -> Vec<String> {
    let text = match std::str::from_utf8(content) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut result = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split_once('|') {
            Some((pkg, proc)) if pkg == package_name => {
                result.push(proc.to_string());
            }
            None if line == package_name => {
                result.push(String::new());
            }
            _ => {}
        }
    }
    result
}

register_module!(HmsPushModule);
register_companion!(server::companion_handler);
