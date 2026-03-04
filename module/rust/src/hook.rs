use std::sync::OnceLock;

use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString, JValue},
    strings::JNIStr,
    sys::JNINativeMethod,
};
use log::debug;

use zygisk_api::api::{V4, ZygiskApi};

// Storage for the original `native_get` function pointer saved by Zygisk.
type NativeGetFn = unsafe extern "C" fn(
    *mut jni::sys::JNIEnv,
    jni::sys::jclass,
    jni::sys::jstring,
    jni::sys::jstring,
) -> jni::sys::jstring;

static ORIG_NATIVE_GET: OnceLock<NativeGetFn> = OnceLock::new();

/// Hook replacement for `SystemProperties.native_get`.
unsafe extern "C" fn my_native_get(
    env: *mut jni::sys::JNIEnv,
    clazz: jni::sys::jclass,
    key_j: jni::sys::jstring,
    def_j: jni::sys::jstring,
) -> jni::sys::jstring {
    // SAFETY: env is a valid JNI pointer provided by the Android runtime.
    let mut jni_env = unsafe { JNIEnv::from_raw(env).expect("invalid JNIEnv") };

    let key: String = if key_j.is_null() {
        String::new()
    } else {
        let js = unsafe { JString::from_raw(key_j) };
        let s: String = jni_env.get_string(&js).map(|s| s.into()).unwrap_or_default();
        // Don't let the JString wrapper drop/delete the local ref we don't own.
        let _ = js.into_raw();
        s
    };

    let spoofed: Option<&str> = match key.as_str() {
        "ro.build.version.emui" => Some("EmotionUI_8.0.0"),
        "ro.build.hw_emui_api_level" => Some("21"),
        _ => None,
    };

    if let Some(value) = spoofed {
        let result = jni_env
            .new_string(value)
            .expect("failed to create JNI string");
        result.into_raw()
    } else {
        match ORIG_NATIVE_GET.get() {
            Some(orig) => unsafe { orig(env, clazz, key_j, def_j) },
            None => def_j,
        }
    }
}

/// Public entry: apply all hooks.
pub fn do_hook(api: &mut ZygiskApi<'_, V4>, env: JNIEnv<'_>) {
    hook_build(&env);
    hook_system_properties(api, env);
}

/// Set android.os.Build.BRAND = "Huawei" and MANUFACTURER = "HUAWEI".
fn hook_build(env: &JNIEnv<'_>) {
    debug!("hook Build");

    // JNIEnv methods in jni 0.21 require &mut self, so we clone for each operation.
    let build_class = {
        let mut e = unsafe { env.unsafe_clone() };
        match e.find_class("android/os/Build") {
            Ok(c) => c,
            Err(e) => {
                debug!("find_class android/os/Build failed: {:?}", e);
                return;
            }
        }
    };

    set_static_string_field(env, &build_class, "BRAND", "Huawei");
    set_static_string_field(env, &build_class, "MANUFACTURER", "HUAWEI");

    debug!("hook Build done");
}

fn set_static_string_field(env: &JNIEnv<'_>, class: &JClass<'_>, field: &str, value: &str) {
    let sig = "Ljava/lang/String;";
    let mut e = unsafe { env.unsafe_clone() };

    let field_id = match e.get_static_field_id(class, field, sig) {
        Ok(id) => id,
        Err(err) => {
            debug!("get_static_field_id {} failed: {:?}", field, err);
            return;
        }
    };
    let new_str = match e.new_string(value) {
        Ok(s) => s,
        Err(err) => {
            debug!("new_string {} failed: {:?}", value, err);
            return;
        }
    };
    let obj = JObject::from(new_str);
    if let Err(err) = e.set_static_field(class, field_id, JValue::Object(&obj)) {
        debug!("set_static_field {} failed: {:?}", field, err);
    }
}

/// Replace `SystemProperties.native_get` via Zygisk's `hookJniNativeMethods`.
fn hook_system_properties(api: &mut ZygiskApi<'_, V4>, env: JNIEnv<'_>) {
    debug!("hook SystemProperties");

    // JNIStr is the type required by hook_jni_native_methods (impl Deref<Target = JNIStr>)
    // SAFETY: literal is valid UTF-8 and contains no interior NUL.
    let class_name: &JNIStr = unsafe {
        JNIStr::from_ptr(b"android/os/SystemProperties\0".as_ptr() as *const _)
    };

    let method_name = b"native_get\0".as_ptr() as *mut std::os::raw::c_char;
    let signature = b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;\0"
        .as_ptr() as *mut std::os::raw::c_char;

    let mut methods = [JNINativeMethod {
        name: method_name,
        signature,
        fnPtr: my_native_get as *mut _,
    }];

    // SAFETY: class_name is nul-terminated, methods slice is valid for the duration of the call.
    unsafe {
        api.hook_jni_native_methods(env, class_name, methods.as_mut_slice());
    }

    // Zygisk writes the original fn pointer back into methods[0].fnPtr
    let orig_ptr = methods[0].fnPtr;
    if !orig_ptr.is_null() {
        let orig_fn: NativeGetFn = unsafe { std::mem::transmute(orig_ptr) };
        let _ = ORIG_NATIVE_GET.set(orig_fn);
        debug!("hook SystemProperties done: {:?}", orig_ptr);
    }
}
