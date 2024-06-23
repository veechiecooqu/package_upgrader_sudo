use std::io::{Error, ErrorKind, Write};

use std::path::Path;

use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

fn copy_with_privilege(src: &Path, dest: &Path) -> std::io::Result<()> {
    let status = Command::new("sudo")
        .args(["cp", src.to_str().unwrap(), dest.to_str().unwrap()])
        .status()
        .map_err(|_e| Error::new(ErrorKind::Other, "Failed to execute sudo command"))?;

    if status.success() {
        // Set the file permissions to -rw-r--r--
        let chmod_status = Command::new("sudo")
            .args(["chmod", "644", dest.to_str().unwrap()])
            .status()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to execute sudo chmod command: {}", e),
                )
            })?;

        if chmod_status.success() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::PermissionDenied,
                "Failed to set file permissions",
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::PermissionDenied,
            "Failed to copy temporary file to destination",
        ))
    }
}
pub fn update_with_fastest_mirrors() {
    // Step 2: Fetch and use the fastest mirrors
    println!("\x1b[0;33mFetching fastest mirrors, please wait...\x1b[0m");
    let output = Command::new("rate-mirrors")
        .args(["arch", "--fetch-mirrors-timeout", "300000"])
        .output()
        .expect("Failed to execute rate-mirrors command");

    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    temp_file
        .write_all(&output.stdout)
        .expect("Failed to write to temporary file");

    let temp_path = temp_file.path();

    let status = Command::new("sudo")
        .args(["chown", "root:root", temp_path.to_str().unwrap()])
        .status()
        .expect("Failed to change ownership of temporary file");

    if status.success() {
        let dest_file = "/etc/pacman.d/mirrorlist";
        copy_with_privilege(temp_path, Path::new(dest_file))
            .expect("Failed to copy temporary file to destination");
    } else {
        eprintln!("Failed to change ownership of temporary file. Cannot update mirrorlist.");
    }
}

pub fn upgrade_packages() {
    // Step 3: Upgrade packages using paru and fisher
    Command::new("paru")
        .stdout(Stdio::inherit())
        .status()
        .expect("Failed to execute paru");

    Command::new("flatpak")
        .args(["update", "--user"])
        .stdout(Stdio::inherit())
        .status()
        .expect("Failed to execute flatpak update");
}
