fn main() {
    #[cfg(windows)]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let project_root = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(std::path::Path::new("."));
        let ico_path = project_root.join("assets").join("famshield.ico");

        if ico_path.exists() {
            let rc_path = std::path::Path::new(&out_dir).join("famshield.rc");
            let obj_path = std::path::Path::new(&out_dir).join("famshield_res.o");
            let ico_str = ico_path.to_str().unwrap_or("").replace('\\', "/");
            let rc_content = format!("1 ICON \"{}\"\n", ico_str);
            if std::fs::write(&rc_path, rc_content).is_ok() {
                if let Ok(status) = std::process::Command::new("windres")
                    .arg("-i")
                    .arg(&rc_path)
                    .arg("-o")
                    .arg(&obj_path)
                    .status()
                {
                    if status.success() {
                        println!("cargo:rustc-link-arg={}", obj_path.display());
                    }
                }
            }
        }
        println!("cargo:rerun-if-changed=../../assets/famshield.ico");
    }
}
