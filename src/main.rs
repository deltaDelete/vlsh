use std::path::Path;
use argh::FromArgs;
use gtk4_layer_shell::KeyboardMode;
use gtk::glib::ExitCode;
use gtk::prelude::*;

const APP_ID : &str = "ru.deltadelete.vlsh";

fn activate(application: &gtk::Application) {
    let args : Args = argh::from_env();
    // Create a normal GTK window
    let window = gtk::ApplicationWindow::new(application);

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
    let video_overlay = video.first_child().unwrap();
    let video_controls = video_overlay.last_child().unwrap();
    video_controls.set_visible(false);

    window.set_child(Some(&video));
    window.present();

    window.set_monitor(args.monitor);
    window.set_fullscreened(true);
}

pub trait VlshExt {
    fn set_monitor(&self, monitor_index : u32);
}

impl VlshExt for gtk::ApplicationWindow {
    fn set_monitor(&self, monitor_index : u32) {
        let monitors = self.surface().display().monitors();
        if monitor_index >= monitors.n_items() {
            println!("monitor id is out of bounds");
            std::process::exit(0);
        }
        let monitor = monitors.item(monitor_index).unwrap().downcast::<gtk::gdk::Monitor>().unwrap();
        let width = monitor.geometry().width();
        let height = monitor.geometry().height();

        self.set_default_width(width);
        self.set_default_height(height);
        gtk4_layer_shell::set_monitor(self, &monitor);
    }
}

fn default_monitor() -> u32 {
    0
}

#[derive(FromArgs)]
/// Set video file as wallpaper
struct Args {
    #[argh(option, short = 'm', default = "default_monitor()")]
    /// id of a monitor
    monitor: u32,
    #[argh(switch, short = 't')]
    /// anchor to top
    anchor_top: bool,
    #[argh(switch, short = 'b')]
    /// anchor to bottom
    anchor_bottom: bool,
    #[argh(switch, short = 'l')]
    /// anchor to left
    anchor_left: bool,
    #[argh(switch, short = 'r')]
    /// anchor to right
    anchor_right: bool,
    #[argh(positional)]
    /// path to a video
    file: String
}

fn main() -> ExitCode {
    let application = gtk::Application::new(Some(APP_ID), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    // Running app with empty args so it doesn't pass real args to GTK
    return application.run_with_args(&Vec::<String>::new());
}