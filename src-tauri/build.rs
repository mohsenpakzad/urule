fn main() {
    tauri_build::build();
    if !cfg!(debug_assertions) && cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.set_manifest_file("Urule.exe.manifest");
        res.compile().unwrap();
    }
}
