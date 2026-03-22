use ini::Ini;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chromium_pwa_positive() {
        let content = "[Desktop Entry]\nExec=/usr/bin/google-chrome-stable --app-id=abcdefg\n";
        assert!(is_chromium_pwa(content));
    }

    #[test]
    fn test_is_chromium_pwa_negative_no_app_id() {
        let content = "[Desktop Entry]\nExec=/usr/bin/google-chrome-stable\n";
        assert!(!is_chromium_pwa(content));
    }

    #[test]
    fn test_is_chromium_pwa_negative_no_desktop_entry() {
        let content = "Exec=/usr/bin/google-chrome-stable --app-id=abcdefg\n";
        assert!(!is_chromium_pwa(content));
    }

    #[test]
    fn test_generate_new_filename_default_profile() {
        let content = "[Desktop Entry]\nName=Google Calendar\nExec=/usr/bin/google-chrome-stable --app-id=abc --profile-directory=Default\n";
        assert_eq!(
            generate_new_filename(content).unwrap(),
            "Google Calendar.desktop"
        );
    }

    #[test]
    fn test_generate_new_filename_work_profile() {
        let content = "[Desktop Entry]\nName=Google Calendar\nExec=/usr/bin/google-chrome-stable --app-id=abc --profile-directory=Work\n";
        assert_eq!(
            generate_new_filename(content).unwrap(),
            "Google Calendar (Work).desktop"
        );
    }

    #[test]
    fn test_generate_new_filename_sanitization() {
        let content = "[Desktop Entry]\nName=Google/Calendar\nExec=/usr/bin/google-chrome-stable --app-id=abc --profile-directory=Default\n";
        assert_eq!(
            generate_new_filename(content).unwrap(),
            "Google-Calendar.desktop"
        );
    }

    #[test]
    fn test_is_companion_file_true() {
        let content =
            "[Desktop Entry]\nX-ChromiumPwaSyncSource=chrome-abc-Default.desktop\n";
        assert!(is_companion_file(content));
    }

    #[test]
    fn test_is_companion_file_false() {
        let content = "[Desktop Entry]\nName=Foo\n";
        assert!(!is_companion_file(content));
    }

    #[test]
    fn test_get_sync_source_present() {
        let content =
            "[Desktop Entry]\nX-ChromiumPwaSyncSource=chrome-abc-Default.desktop\n";
        assert_eq!(
            get_sync_source(content).unwrap(),
            "chrome-abc-Default.desktop"
        );
    }

    #[test]
    fn test_get_sync_source_absent() {
        let content = "[Desktop Entry]\nName=Foo\n";
        assert!(get_sync_source(content).is_none());
    }

    #[test]
    fn test_is_managed_original_true() {
        let content = "[Desktop Entry]\nX-ChromiumPwaSyncManaged=true\n";
        assert!(is_managed_original(content));
    }

    #[test]
    fn test_is_managed_original_false() {
        let content = "[Desktop Entry]\nName=Foo\n";
        assert!(!is_managed_original(content));
    }

    #[test]
    fn test_get_neutered_content() {
        let content =
            "[Desktop Entry]\nIcon=chrome-abc-Default\nStartupWMClass=chrome-abc-Default\n";
        let neutered = get_neutered_content(content).unwrap();
        assert!(!neutered.contains("StartupWMClass"));
        assert!(neutered.contains("NoDisplay=true"));
        assert!(neutered.contains("X-ChromiumPwaSyncManaged=true"));
        assert!(neutered.contains("Icon=chrome-abc-Default"));
    }

    #[test]
    fn test_get_companion_content() {
        let content = "[Desktop Entry]\nIcon=chrome-abc-Default\nStartupWMClass=wrong\n";
        let companion =
            get_companion_content(content, "chrome-abc-Default.desktop").unwrap();
        assert!(companion.contains("StartupWMClass=chrome-abc-Default"));
        assert!(companion.contains("X-ChromiumPwaSyncSource=chrome-abc-Default.desktop"));
        assert!(!companion.contains("NoDisplay"));
    }

    #[test]
    fn test_get_companion_content_from_neutered() {
        let content = "[Desktop Entry]\nIcon=chrome-abc-Default\nNoDisplay=true\nX-ChromiumPwaSyncManaged=true\n";
        let companion =
            get_companion_content(content, "chrome-abc-Default.desktop").unwrap();
        assert!(companion.contains("StartupWMClass=chrome-abc-Default"));
        assert!(companion.contains("X-ChromiumPwaSyncSource=chrome-abc-Default.desktop"));
        assert!(!companion.contains("NoDisplay"));
        assert!(!companion.contains("X-ChromiumPwaSyncManaged"));
    }
}

pub fn is_chromium_pwa(content: &str) -> bool {
    let i = match Ini::load_from_str(content) {
        Ok(i) => i,
        Err(_) => return false,
    };

    let section = match i.section(Some("Desktop Entry")) {
        Some(s) => s,
        None => return false,
    };

    section
        .get("Exec")
        .is_some_and(|exec| exec.contains("--app-id="))
}

pub fn generate_new_filename(content: &str) -> Option<String> {
    let i = Ini::load_from_str(content).ok()?;
    let section = i.section(Some("Desktop Entry"))?;

    let name = section.get("Name")?.trim();
    let exec = section.get("Exec")?.trim();

    let sanitized_name = name.replace('/', "-");

    let profile = exec
        .split_whitespace()
        .find(|arg| arg.starts_with("--profile-directory="))
        .and_then(|arg| arg.split('=').nth(1))
        .unwrap_or("Default");

    if profile == "Default" {
        Some(format!("{}.desktop", sanitized_name))
    } else {
        Some(format!("{} ({}).desktop", sanitized_name, profile))
    }
}

pub fn is_companion_file(content: &str) -> bool {
    get_sync_source(content).is_some()
}

pub fn get_sync_source(content: &str) -> Option<String> {
    let i = Ini::load_from_str(content).ok()?;
    i.section(Some("Desktop Entry"))?
        .get("X-ChromiumPwaSyncSource")
        .map(|s| s.to_string())
}

pub fn is_managed_original(content: &str) -> bool {
    let i = match Ini::load_from_str(content) {
        Ok(i) => i,
        Err(_) => return false,
    };
    i.section(Some("Desktop Entry"))
        .and_then(|s| s.get("X-ChromiumPwaSyncManaged"))
        .is_some_and(|v| v == "true")
}

pub fn get_neutered_content(content: &str) -> Option<String> {
    let mut i = Ini::load_from_str(content).ok()?;
    i.delete_from(Some("Desktop Entry"), "StartupWMClass");
    i.with_section(Some("Desktop Entry"))
        .set("NoDisplay", "true")
        .set("X-ChromiumPwaSyncManaged", "true");
    let mut buf = Vec::new();
    i.write_to(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

pub fn get_companion_content(content: &str, original_filename: &str) -> Option<String> {
    let mut i = Ini::load_from_str(content).ok()?;
    let icon = i
        .section(Some("Desktop Entry"))?
        .get("Icon")?
        .to_string();
    i.with_section(Some("Desktop Entry"))
        .set("StartupWMClass", &icon)
        .set("X-ChromiumPwaSyncSource", original_filename);
    i.delete_from(Some("Desktop Entry"), "NoDisplay");
    i.delete_from(Some("Desktop Entry"), "X-ChromiumPwaSyncManaged");
    let mut buf = Vec::new();
    i.write_to(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

pub fn get_final_filepath(target_path: &Path, current_path: Option<&Path>) -> PathBuf {
    if let Some(cp) = current_path {
        if cp == target_path {
            return target_path.to_path_buf();
        }
    }

    if !target_path.exists() {
        return target_path.to_path_buf();
    }

    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let extension = target_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Prioritize keeping current if it matches pattern
    if let Some(cp) = current_path {
        if let Some(name) = cp.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&format!("{} (", stem))
                && name.ends_with(&format!(").{}", extension))
            {
                return cp.to_path_buf();
            }
        }
    }

    let mut counter = 1;
    loop {
        let new_name = format!("{} ({}).{}", stem, counter, extension);
        let new_path = parent.join(new_name);

        if let Some(cp) = current_path {
            if cp == new_path {
                return new_path;
            }
        }

        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}
