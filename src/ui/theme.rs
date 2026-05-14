//! The responsibility: detect GTK theme preferences used by UI configuration defaults.

use gtk4::{self as gtk, glib::object::ObjectExt};

/// Detects whether GTK currently prefers a dark theme.
///
/// Use this at startup to seed the initial application configuration.
pub fn is_gtk_dark_theme() -> bool {
    match gtk::Settings::default() {
        Some(settings) => {
            let theme_name = settings.property::<String>("gtk-theme-name");
            let is_dark_theme = theme_name.to_lowercase().contains("dark");

            let prefer_dark = settings.property::<bool>("gtk-application-prefer-dark-theme");

            is_dark_theme || prefer_dark
        }
        None => {
            println!("No GTK settings found, defaulting to light mode");
            false
        }
    }
}
