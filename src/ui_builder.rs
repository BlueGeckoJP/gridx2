use gtk4::prelude::{BoxExt, WidgetExt};
use gtk4::{self as gtk, CssProvider, glib};
use gtk4::{Application, gdk};
use std::cell::RefCell;
use std::rc::Rc;

use crate::action_builder::setup_main_window_callbacks;
use crate::config::app_config::AppConfig;
use crate::directory_section::DirectorySection;
use crate::image_cache::ImageCache;
use crate::image_loader::load_and_display_images;
use crate::session::Session;
use crate::ui::main_window::MainWindow;
use crate::ui::widgets::accordion_widget::AccordionWidget;

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

                let dir_entry = match session.dir_entries() {
                    Ok(entries) => match entries
                        .iter()
                        .find(|entry| entry.dir_path == directory_path)
                    {
                        Some(entry) => entry.clone(),
                        None => {
                            eprintln!(
                                "No directory entry found for directory path {directory_path}"
                            );
                            return;
                        }
                    },
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

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Failed to get display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn update_entry(
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
    vbox: gtk::Box,
) -> eyre::Result<()> {
    let sections = DirectorySection::load_sections(session.clone(), app_config.clone())?;
    render_directory_sections(&vbox, &sections, session, app_config, image_cache)
}

fn clear_ui(vbox: &gtk::Box) {
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }
}

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
