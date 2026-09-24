//! Native GNOME frontend for Weylus, built with GTK 4 and libadwaita.
//!
//! This replaces the fltk based startup window on Linux. The fltk backend is
//! still used for the "custom input area" overlay (see `crate::gui`), which
//! runs its own event loop in a dedicated thread.

#![cfg(target_os = "linux")]

use std::cell::{Cell, RefCell};
use std::net::{IpAddr, SocketAddr};
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

use crate::config::{write_config, Config};
use crate::web::Web2UiMessage;

/// Size in px of the rendered QR code.
const QR_SIZE: i32 = 240;

pub fn run(config: &Config, log_receiver: std::sync::mpsc::Receiver<String>) {
    let app = adw::Application::builder()
        .application_id("io.github.h-m-h.Weylus")
        .build();

    let config = config.clone();
    let receiver = Rc::new(RefCell::new(Some(log_receiver)));

    app.connect_activate(move |app| {
        if let Some(receiver) = receiver.borrow_mut().take() {
            build_window(app, &config, receiver);
        }
    });

    app.run();
}

fn build_window(
    app: &adw::Application,
    config: &Config,
    log_receiver: std::sync::mpsc::Receiver<String>,
) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(780)
        .default_height(660)
        .build();

    let toast_overlay = adw::ToastOverlay::new();

    // ---------------------------------------------------------------- header
    let window_title = adw::WindowTitle::new("Weylus", "Not running");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&window_title));

    let button_image =
        gtk4::Image::from_icon_name("media-playback-start-symbolic");
    let button_label = gtk4::Label::new(Some("Start"));
    let button_inner = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    button_inner.append(&button_image);
    button_inner.append(&button_label);
    let button_toggle = gtk4::Button::builder()
        .child(&button_inner)
        .build();
    button_toggle.add_css_class("suggested-action");
    header.pack_start(&button_toggle);

    // ----------------------------------------------------------- left column
    let left = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(18)
        .build();

    let group_server = adw::PreferencesGroup::builder()
        .title("Server")
        .description("How Weylus can be reached from your tablet")
        .build();

    let row_access_code = adw::EntryRow::builder().title("Access Code").build();
    row_access_code.set_text(config.access_code.as_deref().unwrap_or(""));
    row_access_code.set_tooltip_text(Some(
        "Restrict who can control your computer with an access code. This does NOT do any kind \
         of encryption, only run Weylus inside trusted networks! Do NOT reuse any of your \
         passwords! If left blank, no code is required.",
    ));
    group_server.add(&row_access_code);

    let row_bind_address = adw::EntryRow::builder().title("Bind Address").build();
    row_bind_address.set_text(&config.bind_address.to_string());
    group_server.add(&row_bind_address);

    let row_port = adw::EntryRow::builder().title("Port").build();
    row_port.set_text(&config.web_port.to_string());
    row_port.set_input_purpose(gtk4::InputPurpose::Digits);
    group_server.add(&row_port);

    left.append(&group_server);

    let group_options = adw::PreferencesGroup::builder().title("Options").build();
    let (row_auto_start, switch_auto_start) = switch_row(
        "Auto Start",
        "Start the server immediately when Weylus launches",
        config.auto_start,
    );
    group_options.add(&row_auto_start);
    let (row_wayland, switch_wayland) = switch_row(
        "Wayland / PipeWire Support",
        "EXPERIMENTAL! This may crash your desktop! Enables screen capturing on Wayland.",
        config.wayland_support,
    );
    group_options.add(&row_wayland);
    left.append(&group_options);

    let group_hw = adw::PreferencesGroup::builder()
        .title("Hardware Acceleration")
        .description(
            "Quality and stability of hardware encoded video varies greatly among hardware and \
             drivers, so this is disabled by default.",
        )
        .build();
    let (row_vaapi, switch_vaapi) = switch_row(
        "VAAPI",
        "Try to use hardware acceleration through the Video Acceleration API.",
        config.try_vaapi,
    );
    group_hw.add(&row_vaapi);
    let (row_nvenc, switch_nvenc) = switch_row(
        "NVENC",
        "Try to use Nvidia's NVENC to encode the video via GPU.",
        config.try_nvenc,
    );
    group_hw.add(&row_nvenc);
    left.append(&group_hw);

    let group_log = adw::PreferencesGroup::builder().title("Log").build();
    let log_expander = adw::ExpanderRow::builder().title("Log").build();
    let log_view = gtk4::TextView::builder()
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk4::WrapMode::WordChar)
        .build();
    let log_scroll = gtk4::ScrolledWindow::builder()
        .min_content_height(180)
        .child(&log_view)
        .build();
    log_expander.add_row(&log_scroll);
    group_log.add(&log_expander);
    left.append(&group_log);

    let clamp = adw::Clamp::builder().maximum_size(460).build();
    clamp.set_child(Some(&left));
    let scroller = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .hexpand(true)
        .child(&clamp)
        .build();

    // ---------------------------------------------------------- right column
    let right = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .build();
    right.set_size_request(270, -1);

    let status_page = adw::StatusPage::builder()
        .icon_name("network-wireless-symbolic")
        .title("Not connected")
        .description("Press Start to make Weylus available to your tablet.")
        .vexpand(true)
        .build();

    let qr_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .build();
    qr_box.set_visible(false);

    let qr_picture = gtk4::Picture::builder()
        .content_fit(gtk4::ContentFit::Contain)
        .width_request(QR_SIZE)
        .height_request(QR_SIZE)
        .build();
    qr_picture.add_css_class("card");

    let group_connection = adw::PreferencesGroup::builder()
        .title("Connection")
        .description("Scan the code or open the address on your tablet")
        .build();
    let row_url = adw::ActionRow::builder().title("Address").build();
    let button_copy = gtk4::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy address")
        .valign(gtk4::Align::Center)
        .build();
    button_copy.add_css_class("flat");
    row_url.add_suffix(&button_copy);
    group_connection.add(&row_url);

    qr_box.append(&qr_picture);
    qr_box.append(&group_connection);

    right.append(&status_page);
    right.append(&qr_box);

    // ---------------------------------------------------------------- layout
    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(18)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&scroller);
    content.append(&right);

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&content));

    toast_overlay.set_child(Some(&toolbar_view));
    window.set_content(Some(&toast_overlay));

    // ------------------------------------------------------------------- log
    // GTK widgets are not `Send`, so instead of pushing log messages from the
    // logging thread the buffer is drained from the main loop.
    {
        let buffer = log_view.buffer();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            while let Ok(message) = log_receiver.try_recv() {
                let mut end = buffer.end_iter();
                buffer.insert(&mut end, &message);
                let mut end = buffer.end_iter();
                log_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
            }
            gtk4::glib::ControlFlow::Continue
        });
    }

    // Channel used by the web server to report that uinput is inaccessible.
    let (uinput_sender, uinput_receiver) = std::sync::mpsc::channel::<()>();
    {
        let overlay = toast_overlay.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if uinput_receiver.try_recv().is_ok() {
                let dialog = adw::AlertDialog::builder()
                    .heading("UInput inaccessible!")
                    .body(std::include_str!("strings/uinput_error.txt"))
                    .build();
                dialog.add_response("ok", "OK");
                dialog.set_default_response(Some("ok"));
                let parent = overlay
                    .root()
                    .and_then(|root| root.downcast::<gtk4::Window>().ok());
                dialog.present(parent.as_ref());
            }
            gtk4::glib::ControlFlow::Continue
        });
    }

    // ------------------------------------------------------------ start/stop
    let weylus = Rc::new(RefCell::new(crate::weylus::Weylus::new()));
    let is_running = Rc::new(Cell::new(false));
    let config = Rc::new(RefCell::new(config.clone()));

    let toggle: Rc<dyn Fn()> = {
        let weylus = weylus.clone();
        let is_running = is_running.clone();
        let config = config.clone();
        let button = button_toggle.clone();
        let button_image = button_image.clone();
        let button_label = button_label.clone();
        let window_title = window_title.clone();
        let status_page = status_page.clone();
        let qr_box = qr_box.clone();
        let qr_picture = qr_picture.clone();
        let row_url = row_url.clone();
        let overlay = toast_overlay.clone();
        let row_access_code = row_access_code.clone();
        let row_bind_address = row_bind_address.clone();
        let row_port = row_port.clone();
        let switch_auto_start = switch_auto_start.clone();
        let switch_wayland = switch_wayland.clone();
        let switch_vaapi = switch_vaapi.clone();
        let switch_nvenc = switch_nvenc.clone();
        let uinput_sender = uinput_sender.clone();

        Rc::new(move || {
            if is_running.get() {
                weylus.borrow_mut().stop();
                is_running.set(false);
                button_label.set_text("Start");
                button_image.set_icon_name(Some("media-playback-start-symbolic"));
                button.add_css_class("suggested-action");
                window_title.set_subtitle("Not running");
                status_page.set_visible(true);
                qr_box.set_visible(false);
                return;
            }

            {
                let mut cfg = config.borrow_mut();
                let access_code = row_access_code.text().to_string();
                cfg.access_code = if access_code.is_empty() {
                    None
                } else {
                    Some(access_code)
                };
                match row_bind_address.text().parse::<IpAddr>() {
                    Ok(addr) => cfg.bind_address = addr,
                    Err(_) => {
                        show_toast(&overlay, "Invalid bind address.");
                        return;
                    }
                }
                match row_port.text().parse::<u16>() {
                    Ok(port) => cfg.web_port = port,
                    Err(_) => {
                        show_toast(&overlay, "Invalid port.");
                        return;
                    }
                }
                cfg.auto_start = switch_auto_start.is_active();
                cfg.wayland_support = switch_wayland.is_active();
                cfg.try_vaapi = switch_vaapi.is_active();
                cfg.try_nvenc = switch_nvenc.is_active();
            }

            let cfg = config.borrow().clone();
            let uinput_sender = uinput_sender.clone();
            let started = weylus.borrow_mut().start(&cfg, move |message| match message {
                Web2UiMessage::UInputInaccessible => {
                    let _ = uinput_sender.send(());
                }
            });

            if !started {
                show_toast(&overlay, "Failed to start the server.");
                return;
            }

            write_config(&cfg);
            is_running.set(true);
            button_label.set_text("Stop");
            button_image.set_icon_name(Some("media-playback-stop-symbolic"));
            button.remove_css_class("suggested-action");
            window_title.set_subtitle(&format!("Listening on port {}", cfg.web_port));

            let address = guess_address(&cfg);
            row_url.set_subtitle(&address);
            if let Some(texture) = qr_texture(&url_with_access_code(&cfg, &address)) {
                qr_picture.set_paintable(Some(&texture));
            }
            status_page.set_visible(false);
            qr_box.set_visible(true);
        })
    };

    {
        let toggle = toggle.clone();
        button_toggle.connect_clicked(move |_| toggle());
    }

    {
        let url = row_url.clone();
        let overlay = toast_overlay.clone();
        button_copy.connect_clicked(move |_| {
            let address = url
                .subtitle()
                .map(|subtitle| subtitle.to_string())
                .unwrap_or_default();
            if let Some(display) = gtk4::gdk::Display::default() {
                display.clipboard().set_text(&address);
            }
            overlay.add_toast(adw::Toast::new("Address copied."));
        });
    }

    {
        let weylus = weylus.clone();
        let is_running = is_running.clone();
        window.connect_close_request(move |_| {
            if is_running.get() {
                weylus.borrow_mut().stop();
                is_running.set(false);
            }
            gtk4::glib::Propagation::Proceed
        });
    }

    window.present();

    if config.borrow().auto_start {
        toggle();
    }
}

fn switch_row(title: &str, subtitle: &str, active: bool) -> (adw::ActionRow, gtk4::Switch) {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    let switch = gtk4::Switch::builder()
        .valign(gtk4::Align::Center)
        .active(active)
        .build();
    row.add_suffix(&switch);
    row.set_activatable_widget(Some(&switch));
    (row, switch)
}

fn show_toast(overlay: &adw::ToastOverlay, message: &str) {
    overlay.add_toast(adw::Toast::new(message));
}

/// Best effort guess of the address a client should connect to.
fn guess_address(config: &Config) -> String {
    let mut socket = SocketAddr::new(config.bind_address, config.web_port);
    if socket.ip().is_unspecified() {
        let mut ips = Vec::<IpAddr>::new();
        for iface in pnet_datalink::interfaces()
            .iter()
            .filter(|iface| iface.is_up() && !iface.is_loopback())
        {
            for network in &iface.ips {
                if network.is_ipv4() == socket.ip().is_ipv4() {
                    ips.push(network.ip());
                }
            }
        }
        if !ips.is_empty() {
            socket.set_ip(ips[0]);
        }
    }
    format!("http://{}", socket)
}

fn url_with_access_code(config: &Config, address: &str) -> String {
    let mut url = address.to_string();
    if let Some(access_code) = &config.access_code {
        url.push_str("?access_code=");
        url.push_str(
            &percent_encoding::utf8_percent_encode(
                access_code,
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string(),
        );
    }
    url
}

fn qr_texture(data: &str) -> Option<gtk4::gdk::Texture> {
    use image::ImageFormat;
    use qrcode::QrCode;

    let code = QrCode::new(data).ok()?;
    let buffer = code.render::<image::Luma<u8>>().build();
    let image = image::DynamicImage::ImageLuma8(buffer)
        .resize_exact(QR_SIZE as u32, QR_SIZE as u32, image::imageops::FilterType::Nearest);
    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .ok()?;
    gtk4::gdk::Texture::from_bytes(&gtk4::glib::Bytes::from(&bytes)).ok()
}
