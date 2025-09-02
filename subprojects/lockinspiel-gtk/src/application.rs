use adw::prelude::AdwDialogExt;
use gettextrs::gettext;
use tracing::{debug, info};

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::window::LockinspielApplicationWindow;
use crate::{APP_ID, PKGDATADIR, PROFILE, VERSION};

const APP_NAME: &str = "Lockinspiel";

mod imp {
    use super::*;
    use glib::WeakRef;
    use std::cell::OnceCell;

    #[derive(Debug, Default)]
    pub struct LockinspielApplication {
        pub window: OnceCell<WeakRef<LockinspielApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LockinspielApplication {
        const NAME: &'static str = "LockinspielApplication";
        type Type = super::LockinspielApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for LockinspielApplication {}

    impl ApplicationImpl for LockinspielApplication {
        fn activate(&self) {
            debug!("AdwApplication<LockinspielApplication>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = LockinspielApplicationWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("AdwApplication<LockinspielApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl AdwApplicationImpl for LockinspielApplication {}
    impl GtkApplicationImpl for LockinspielApplication {}
}

glib::wrapper! {
    pub struct LockinspielApplication(ObjectSubclass<imp::LockinspielApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl LockinspielApplication {
    fn main_window(&self) -> LockinspielApplicationWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();
        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/live/Lockinspiel/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn developers() -> Vec<&'static str> {
        // Authors are defined in Cargo.toml
        env!("CARGO_PKG_AUTHORS").split(":").collect()
    }

    fn show_about_dialog(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name(APP_NAME)
            .application_icon(APP_ID)
            .developer_name("Isaac Mills")
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/StratusFearMe21/lockinspiel/")
            .issue_url("https://github.com/StratusFearMe21/lockinspiel/issues")
            .version(VERSION)
            .translator_credits(gettext("translator-credits"))
            .developers(Self::developers())
            .artists(vec!["Isaac Mills"])
            .build();

        dialog.present(Some(&self.main_window()));
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("Lockinspiel ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for LockinspielApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/live/Lockinspiel")
            .build()
    }
}
