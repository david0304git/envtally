// release 版本不開主控台視窗（僅 Windows 有效，macOS 無害）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    envtally_lib::run()
}
