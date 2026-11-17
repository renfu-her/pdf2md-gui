fn main() {
    // Note: no winres here on purpose. tauri-build already generates the
    // Windows resource file (exe icon from `bundle.icon`, version info from
    // the config); adding a second resource file breaks the link with
    // CVT1100 duplicate resource errors.
    tauri_build::build()
}
