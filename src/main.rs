use chrono::{Duration, Utc};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    // Step 1: Print message
    println!("Updating packages, please wait...");

    let mirrorlist_path = "/etc/pacman.d/mirrorlist";
    let can_update = if !Path::new(mirrorlist_path).exists() {
        true
    } else {
        let metadata = fs::metadata(mirrorlist_path).unwrap();
        let last_modified = metadata.modified().unwrap();

        let duration_since_modified = Utc::now().signed_duration_since::<Utc>(last_modified.into());

        duration_since_modified > Duration::days(1)
    };

    // Run Step 2 only if the condition is met
    if can_update {
        update_with_fastest_mirrors();
    }

    // Step 3: Always run the package upgrade
    upgrade_packages();
}

fn update_with_fastest_mirrors() {
    // Step 2: Fetch and use the fastest mirrors
    println!("\x1b[0;33mFetching fastest mirrors, please wait...\x1b[0m");
    let status = Command::new("doas")
        .args([
            "cp",
            "-b",
            "--no-preserve=mode,ownership",
            &(format!(
                "rate-mirrors arch --fetch-mirrors-timeout {} | psub",
                300000
            )),
            "/etc/pacman.d/mirrorlist",
        ])
        .status()
        .expect("Failed to execute command");

    if status.success() {
        println!("\x1b[0;32mUsing fastest mirrors.\x1b[0m");
    } else {
        println!("\x1b[0;31mFetch failed. Using previously fetched mirrors.\x1b[0m");
    }
}

fn upgrade_packages() {
    // Step 3: Upgrade packages using paru and fisher
    Command::new("paru")
        .stdout(Stdio::inherit())
        .status()
        .expect("Failed to execute paru");

    Command::new("fish")
        .args(["-c", "fisher update"])
        .stdout(Stdio::inherit())
        .status()
        .expect("Failed to execute fisher update");
}
