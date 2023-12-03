use logind_dbus::LoginManager;
mod updater;

fn main() {
    // Step 1: Print message
    println!("Updating packages, please wait...");
    let mut suspend_lock = Err("Did not work");
    if let Ok(login_manager) = LoginManager::new() {
        suspend_lock = Ok(login_manager
            .connect()
            .inhibit_suspend("Package Upgrader", "Prevents sleep during upgrade"));
    }
    if suspend_lock.is_ok() {
        println!("Idle inhibit ready");
    }

    let mirrorlist_path = "/etc/pacman.d/mirrorlist";
    let can_update = if !std::path::Path::new(mirrorlist_path).exists() {
        true
    } else {
        let metadata = std::fs::metadata(mirrorlist_path).unwrap();
        let last_modified = metadata.modified().unwrap();

        let duration_since_modified =
            chrono::Utc::now().signed_duration_since::<chrono::Utc>(last_modified.into());

        duration_since_modified > chrono::Duration::days(1)
    };

    // Run Step 2 only if the condition is met
    if can_update {
        updater::update_with_fastest_mirrors();
    }

    // Step 3: Always run the package upgrade
    updater::upgrade_packages();
}
