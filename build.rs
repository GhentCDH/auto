fn main() {
    println!("cargo:rerun-if-changed=frontend");

    if std::env::var("SKIP_FRONTEND_BUILD").is_ok() {
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        return;
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let frontend_dir = format!("{manifest_dir}/frontend");

    if !std::path::Path::new(&frontend_dir).exists() {
        println!("cargo:warning=frontend/ directory not found, skipping frontend build");
        return;
    }

    let status = std::process::Command::new("bun")
        .args(["run", "build"])
        .current_dir(&frontend_dir)
        .status()
        .expect("bun not found — install Bun (https://bun.sh) to build the frontend");

    if !status.success() {
        panic!("frontend build failed");
    }
}
