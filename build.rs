use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=crates/todoapp-frontend/src");
    println!("cargo:rerun-if-changed=crates/todoapp-frontend/assets");
    println!("cargo:rerun-if-changed=crates/todoapp-frontend/Dioxus.toml");
    println!("cargo:rerun-if-changed=crates/todoapp-frontend/Cargo.toml");

    let frontend_dir = Path::new("crates/todoapp-frontend");
    let dist_dir = frontend_dir.join("dist");

    // Check if dx is installed
    let dx_check = Command::new("dx").arg("--version").output();

    if dx_check.is_err() {
        eprintln!("Warning: 'dx' CLI not found. Frontend will not be built.");
        eprintln!("Install with: cargo install dioxus-cli");
        eprintln!("Skipping frontend build...");
        return;
    }

    println!("Building frontend with dx...");

    // Build the frontend
    let status = Command::new("dx")
        .arg("build")
        .arg("--release")
        .arg("--platform")
        .arg("web")
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to execute dx build");

    if !status.success() {
        panic!("Frontend build failed!");
    }

    // Copy built files from target/dx to dist directory
    let dx_output = Path::new("target/dx/todoapp-frontend/release/web/public");

    if dx_output.exists() {
        // Remove old dist directory if it exists
        if dist_dir.exists() {
            fs::remove_dir_all(&dist_dir).expect("Failed to remove old dist directory");
        }

        // Create dist directory
        fs::create_dir_all(&dist_dir).expect("Failed to create dist directory");

        // Copy all files from dx output to dist
        copy_dir_all(dx_output, &dist_dir).expect("Failed to copy frontend build to dist");

        println!("Frontend copied to dist directory successfully!");
    } else {
        panic!("Frontend build output not found at {:?}", dx_output);
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
