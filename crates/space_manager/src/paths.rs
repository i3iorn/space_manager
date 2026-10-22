//! Where the game keeps its files.
//!
//! **A game that writes into its own install directory is a game that breaks
//! the moment it is installed read-only**, and a game that writes beside its
//! own executable is a game whose save file turns up in whatever folder the
//! player happened to run it from. The platform's own configuration directory
//! is the answer, and it is spelled differently on every platform.
//!
//! **It is the binary that knows about the machine.** The interface is handed a
//! path and writes to it; it does not decide where that path is. The design
//! says so too: the save format's directory layout is the platform's standard
//! config directory, and not the working directory.

use std::path::PathBuf;

/// The directory the game's files live in.
const APP_DIRECTORY: &str = "space_manager";

/// The file the campaign is saved to.
const CAMPAIGN_FILE: &str = "campaign.json";

/// The platform's configuration directory for the game.
///
/// **Returns `None` when the platform will not say where its home directory
/// is**, which is rare but real. The caller falls back to a file beside the
/// game rather than refusing to save at all — a save in the wrong place is
/// better than no save.
#[must_use]
pub fn config_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        // **The roaming application data directory**, which is where Windows
        // puts a user's settings. The local variant would be wrong: a setting
        // that follows the user between machines is a setting, and a cache is
        // not.
        std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|home| PathBuf::from(home).join("AppData").join("Roaming"))
        })?
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        })?
    } else {
        // **The XDG base directory**, which is the freedesktop.org standard
        // and which every Linux desktop honours.
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?
    };
    Some(base.join(APP_DIRECTORY))
}

/// The file the campaign is saved to.
#[must_use]
pub fn campaign_file() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(CAMPAIGN_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_campaign_file_is_named_after_the_game() {
        // **The name is what the player sees in the notice**, so it has to say
        // what it is and where it belongs.
        let Some(path) = campaign_file() else {
            // **A machine with no home directory is a machine with no config
            // directory**, and the caller falls back. That is not a failure.
            return;
        };
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("campaign.json")
        );
        assert_eq!(
            path.parent()
                .and_then(|dir| dir.file_name())
                .and_then(|name| name.to_str()),
            Some(APP_DIRECTORY)
        );
    }

    #[test]
    fn the_config_directory_is_not_in_the_working_directory() {
        // **A relative path is a path that depends on where the player
        // happened to be**, and a save file that moves is a save file that is
        // lost.
        let Some(dir) = config_dir() else {
            return;
        };
        assert!(
            dir.is_absolute(),
            "the configuration directory is absolute: {}",
            dir.display()
        );
    }
}
