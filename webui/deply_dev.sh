bun run clean
bun run build
adb push dist /data/local/tmp
adb shell su -c "rm -rf /data/adb/modules/zygisk_hmspush/webroot/*"
adb shell su -c "mv /data/local/tmp/dist/* /data/adb/modules/zygisk_hmspush/webroot/"
adb shell su -c "am start -n me.weishu.kernelsu/me.weishu.kernelsu.ui.webui.WebUIActivity -d \"kernelsu://webui/zygisk_hmspush\" -e id zygisk_hmspush"