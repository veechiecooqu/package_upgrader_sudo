use chrono::Utc;
use logind_zbus::manager::{InhibitType, ManagerProxyBlocking};
mod updater;

fn main() {
    // Step 1: Print message
    println!("Updating packages, please wait...");

    // This makes the suspend_lock live as long as main
    let mut suspend_lock = Err(::zbus::Error::MissingField);
    // Step 1: Grab a system connection (I will never do async) because inhibitors live in system
    if let Ok(conn) = ::zbus::blocking::Connection::system() {
        // Step 2: Grab a manager proxy
        if let Ok(manager_proxy) = ManagerProxyBlocking::new(&conn) {
            // Step 3: Finally get your suspend inhibitors.
            suspend_lock = manager_proxy.inhibit(
                InhibitType::Sleep,
                "Package Upgrader",
                "Prevents sleep during upgrade",
                "delay",
            );
        }
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
            Utc::now().signed_duration_since::<Utc>(chrono::DateTime::from(last_modified));

        duration_since_modified > chrono::Duration::days(1)
    };
    // Run Step 2 only if the condition is met
    if can_update {
        updater::update_with_fastest_mirrors();
    }

    // Step 3: Always run the package upgrade
    updater::upgrade_packages();
}
