//! The responsibility: build and refresh the main GTK interface.

use gtk4::prelude::{BoxExt, WidgetExt};
use gtk4::{self as gtk, CssProvider, glib};
use gtk4::{Application, gdk};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::app_config::AppConfig;
use crate::directory::browser::DirectoryBrowser;
use crate::directory::section::DirectorySection;
use crate::image::cache::ImageCache;
use crate::image::loader::load_and_display_images;
use crate::session::Session;
use crate::ui::action_builder::setup_main_window_callbacks;
use crate::ui::main_window::MainWindow;
use crate::ui::widgets::accordion_widget::AccordionWidget;

/// Builds the top-level UI and connects its callbacks.
///
/// Use this from `Application::connect_activate`; it owns the first render of the GTK shell.
pub fn build_ui(
    app: &Application,
    app_config: AppConfig,
    image_cache: ImageCache,
    session: Session,
) {
    load_css();

    let main_window = MainWindow::new(app);
    setup_main_window_callbacks(
        &main_window,
        session.clone(),
        app_config.clone(),
        image_cache.clone(),
    );
}

/// Creates an accordion placeholder for one directory section.
///
/// Thumbnails are loaded lazily by the expand handler so initial rendering does not decode every
/// image in every directory.
pub fn create_blank_accordion_widget(
    vbox: &gtk::Box,
    image_count: usize,
    title: &str,
    directory_path: String,
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
) -> eyre::Result<()> {
    let dark_mode = match app_config.get() {
        Ok(config) => config.dark_mode,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            false
        }
    };

    let accordion_widget = Rc::new(RefCell::new(AccordionWidget::new(title, dark_mode)?));
    vbox.append(&accordion_widget.borrow().widget);

    setup_accordion_expand_handler(
        directory_path,
        image_count,
        accordion_widget,
        session,
        app_config,
        image_cache,
    );

    Ok(())
}

/// Wires lazy thumbnail loading to a single accordion expansion event.
///
/// This resolves the backing directory entry from session state, resets the visible grid, and starts
/// image loading when the section is expanded.
fn setup_accordion_expand_handler(
    directory_path: String,
    image_count: usize,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
) {
    accordion_widget
        .clone()
        .borrow()
        .connect_expanded(move |is_expanded| {
            if is_expanded {
                let session = session.clone();
                let app_config = app_config.clone();
                let image_cache = image_cache.clone();
                let accordion_widget = accordion_widget.clone();

                let mut overlays = Vec::new();
                let thumbnail_size = match app_config.get() {
                    Ok(config) => config.thumbnail_size as i32,
                    Err(e) => {
                        eprintln!("Failed to get app config: {e}");
                        300
                    }
                };

                reset_accordion_view(
                    &accordion_widget,
                    &mut overlays,
                    thumbnail_size,
                    image_count,
                );

                let dir_entry = match session.find_dir_entry(&directory_path) {
                    Ok(Some(entry)) => entry,
                    Ok(None) => {
                        eprintln!("No directory entry found for directory path {directory_path}");
                        return;
                    }
                    Err(e) => {
                        eprintln!("Failed to get directory entries: {e}");
                        return;
                    }
                };

                glib::spawn_future_local(async move {
                    load_and_display_images(
                        dir_entry,
                        image_cache,
                        app_config.clone(),
                        accordion_widget,
                        overlays,
                    )
                    .await;
                });
            }
        });
}

/// Clears and prepares thumbnail overlay slots before a directory is loaded.
///
/// The prepared overlays are populated later by the image grid presenter.
fn reset_accordion_view(
    accordion_widget: &Rc<RefCell<AccordionWidget>>,
    overlays: &mut Vec<gtk::Overlay>,
    thumbnail_size: i32,
    image_count: usize,
) {
    let accordion_widget = accordion_widget.borrow();

    while let Some(child) = accordion_widget.flow_box.first_child() {
        accordion_widget.flow_box.remove(&child);
    }

    for _ in 0..image_count {
        let fixed_size_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        fixed_size_container.set_size_request(thumbnail_size, thumbnail_size);
        fixed_size_container.set_halign(gtk::Align::Center);
        fixed_size_container.set_valign(gtk::Align::Center);

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&fixed_size_container));

        //accordion_widget.flow_box.append(&overlay);
        overlays.push(overlay);
    }

    accordion_widget.progress_bar.set_fraction(0.0);
    accordion_widget.progress_bar.set_visible(true);
}

/// Registers the application CSS for the current GTK display.
fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Failed to get display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Reloads directory sections for the current session and renders them into the main container.
///
/// Use this after selecting a directory or changing settings that affect directory discovery.
pub fn update_entry(
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
    vbox: gtk::Box,
) -> eyre::Result<()> {
    let sections = DirectoryBrowser::new(session.clone(), app_config.clone()).load_sections()?;
    render_directory_sections(&vbox, &sections, session, app_config, image_cache)
}

/// Removes all currently rendered directory sections from the main container.
fn clear_ui(vbox: &gtk::Box) {
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }
}

/// Renders section view models as lazy-loading accordion widgets.
fn render_directory_sections(
    vbox: &gtk::Box,
    sections: &[DirectorySection],
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
) -> eyre::Result<()> {
    clear_ui(vbox);

    for section in sections {
        create_blank_accordion_widget(
            vbox,
            section.image_count(),
            section.title(),
            section.directory_path(),
            session.clone(),
            app_config.clone(),
            image_cache.clone(),
        )?;
    }

    Ok(())
}
