use std::fs;
use std::path::Path;

fn main() {
    // Ensure web/dist exists so rust-embed can compile even in a fresh CI checkout
    // without running pnpm build first.
    let dist_dir = Path::new("../../../web/dist");
    if !dist_dir.exists() {
        let _ = fs::create_dir_all(dist_dir.join("assets"));
        let index_file = dist_dir.join("index.html");
        if !index_file.exists() {
            let fallback_html = r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>FamShield Admin</title></head><body><h1>FamShield Web Admin</h1><p>Placeholder bundle. Run <code>pnpm --prefix web run build</code> to generate production assets.</p></body></html>"#;
            let _ = fs::write(index_file, fallback_html);
        }
    }
    println!("cargo:rerun-if-changed=../../../web/dist");
}
