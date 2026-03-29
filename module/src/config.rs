pub const SPOOF_SYSTEM_PROPERTIES: &[(&str, &str)] = &[
    ("ro.build.version.emui", "EmotionUI_8.0.0"),
    ("ro.build.hw_emui_api_level", "21"),
];

pub const SPOOF_BUILD_PROPERTIES: &[(&str, &str)] =
    &[("BRAND", "Huawei"), ("MANUFACTURER", "HUAWEI")];

pub const CONFIG_PATH: &str = "/data/adb/hmspush/app.conf";

pub const HMSPUSH_PACKAGE_NAME: &str = "one.yufz.hmspush";

pub const SPOOF_HMSPUSH_PROPERTIES: &[(&str, &str)] = &[("hmspush.zygisk.enabled", "true")];
