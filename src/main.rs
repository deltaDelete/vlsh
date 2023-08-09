use std::path::Path;
use gtk4_layer_shell::{KeyboardMode};
use gtk::{Application, gio};
use gtk::gio::{ApplicationCommandLine};
use gtk::glib::{ExitCode, OptionArg, OptionFlags, VariantDict};
use gtk::prelude::*;

const APP_ID: &str = "ru.deltadelete.vlsh";

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn activate(application: &Application, args: &Args) {
    // Create a normal GTK window
    let window = gtk::Window::builder()
        .application(application)
        .build();

    // Before the window is first realized, set it up to be a layer surface
    gtk4_layer_shell::init_for_window(&window);

    // Display below windows
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Background);

    gtk4_layer_shell::set_namespace(&window, "vlsh");

    gtk4_layer_shell::set_keyboard_mode(&window, KeyboardMode::None);

    // Margins and anchors
    gtk4_layer_shell::set_margin(&window, gtk4_layer_shell::Edge::Left, 0);
    gtk4_layer_shell::set_margin(&window, gtk4_layer_shell::Edge::Right, 0);
    gtk4_layer_shell::set_margin(&window, gtk4_layer_shell::Edge::Top, 0);
    gtk4_layer_shell::set_margin(&window, gtk4_layer_shell::Edge::Bottom, 0);

    gtk4_layer_shell::set_anchor(&window, gtk4_layer_shell::Edge::Top, args.anchor_top);
    gtk4_layer_shell::set_anchor(&window, gtk4_layer_shell::Edge::Bottom, args.anchor_bottom);
    gtk4_layer_shell::set_anchor(&window, gtk4_layer_shell::Edge::Left, args.anchor_left);
    gtk4_layer_shell::set_anchor(&window, gtk4_layer_shell::Edge::Right, args.anchor_right);

    gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    gtk4_layer_shell::set_exclusive_zone(&window, 1);

    // The 'main' component
    let video = gtk::Video::builder()
        .loop_(true)
        .autoplay(true)
        .margin_top(0)
        .margin_bottom(0)
        .margin_start(0)
        .margin_end(0)
        .hexpand(false)
        .vexpand(false)
        .build();

    // Settings file from arguments
    video.set_filename(Some(&Path::new(&args.file)));

    // Hiding controls
    let video_overlay = video.first_child()
        .expect("Failed to get video controls overlay");
    let video_controls = video_overlay.last_child()
        .expect("Failed to get video controls");
    video_controls.set_visible(false);

    window.set_child(Some(&video));
    window.present();

    window.set_monitor(args.monitor);
    window.set_fullscreened(true);
}

pub trait VlshExt {
    fn set_monitor(&self, monitor_index: u32);
}

impl VlshExt for gtk::Window {
    fn set_monitor(&self, monitor_index: u32) {
        let monitors = self.surface().display().monitors();
        if monitor_index >= monitors.n_items() {
            println!("monitor id is out of bounds");
            std::process::exit(0);
        }
        let monitor = monitors.iter::<gtk::gdk::Monitor>()
            .nth(monitor_index as usize)
            .expect(&format!("Failed to get monitor with index {}", monitor_index))
            .expect("Data mutated during iteration");

        let width = monitor.geometry().width();
        let height = monitor.geometry().height();

        self.set_default_width(width);
        self.set_default_height(height);
        gtk4_layer_shell::set_monitor(self, &monitor);
    }
}

struct Args {
    /// id of a monitor
    monitor: u32,
    /// anchor to top
    anchor_top: bool,
    /// anchor to bottom
    anchor_bottom: bool,
    /// anchor to left
    anchor_left: bool,
    /// anchor to right
    anchor_right: bool,
    /// path to a video
    file: String,
}

impl Args {
    pub fn from_variant_dict(options: &VariantDict, file: Option<String>) -> Self {
        let monitor_result = options.lookup::<i32>("monitor");
        let monitor_option = monitor_result.expect("Expected parameter \"monitor\" to be integer value");
        let monitor = monitor_option.unwrap_or(0);

        let args: Args = Args {
            monitor: monitor as u32,
            anchor_top: options.contains("anchor_top"),
            anchor_bottom: options.contains("anchor_bottom"),
            anchor_left: options.contains("anchor_left"),
            anchor_right: options.contains("anchor_right"),
            file: file.unwrap_or(String::from("")),
        };

        return args;
    }
}

fn main() -> ExitCode {
    let application = Application::new(
        Some(APP_ID),
        gio::ApplicationFlags::HANDLES_COMMAND_LINE,
    );

    application.connect_activate(|_app| {
        println!("Running in the background");
    });

    application.add_main_option(
        "version",
        gtk::glib::Char::from(b'v'),
        OptionFlags::NONE,
        OptionArg::None,
        "get version",
        None,
    );

    application.add_main_option(
        "anchor_top",
        gtk::glib::Char::from(b't'),
        OptionFlags::NONE,
        OptionArg::None,
        "anchor to top",
        None,
    );

    application.add_main_option(
        "anchor_bottom",
        gtk::glib::Char::from(b'b'),
        OptionFlags::NONE,
        OptionArg::None,
        "anchor to bottom",
        None,
    );

    application.add_main_option(
        "anchor_left",
        gtk::glib::Char::from(b'l'),
        OptionFlags::NONE,
        OptionArg::None,
        "anchor to left",
        None,
    );

    application.add_main_option(
        "anchor_right",
        gtk::glib::Char::from(b'r'),
        OptionFlags::NONE,
        OptionArg::None,
        "anchor to right",
        None,
    );

    application.add_main_option(
        "monitor",
        gtk::glib::Char::from(b'm'),
        OptionFlags::NONE,
        OptionArg::Int,
        "set the monitor on which the video will be displayed",
        Some("<monitor id>"),
    );

    application.set_option_context_parameter_string(Some("<file>"));
    application.set_option_context_summary(Some("summary"));

    application.connect_handle_local_options(handle_options);
    application.connect_command_line(handle_cli);

    return application.run();
}

fn handle_options(_app: &Application, options: &VariantDict) -> i32 {
    if options.contains("version") {
        println!("{}", built_info::GIT_VERSION.unwrap());
        return 0;
    }
    return -1;
}

fn handle_cli(app: &Application, cli: &ApplicationCommandLine) -> i32 {
    let options = cli.options_dict();
    let file = String::from(cli.arguments().last().unwrap().to_str().unwrap());

    let args = Args::from_variant_dict(&options, Some(file));

    activate(app, &args);

    return -1;
}