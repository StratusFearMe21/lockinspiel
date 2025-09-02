mod application;
mod window;

use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};

use self::application::LockinspielApplication;

#[cfg(not(meson))]
pub const APP_ID: &str = "";
#[cfg(not(meson))]
pub const GETTEXT_PACKAGE: &str = "";
#[cfg(not(meson))]
pub const LOCALEDIR: &str = "";
#[cfg(not(meson))]
pub const PKGDATADIR: &str = "";
#[cfg(not(meson))]
pub const PROFILE: &str = "";
#[cfg(not(meson))]
pub const RESOURCES_FILE: &str = "";
#[cfg(not(meson))]
pub const VERSION: &str = "";

#[cfg(meson)]
include!(concat!(env!("MESON_BUILD_ROOT"), "/src/config.rs"));

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("Lockinspiel"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = LockinspielApplication::default();
    app.run()
}
