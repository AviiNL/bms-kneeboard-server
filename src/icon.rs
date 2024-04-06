use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

pub fn start(listen: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoopBuilder::new().build().unwrap();

    let mut listen = listen;
    if listen.ip().is_unspecified() {
        if listen.is_ipv4() {
            listen.set_ip(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST));
        }
        if listen.is_ipv6() {
            listen.set_ip(std::net::IpAddr::V6(Ipv6Addr::LOCALHOST));
        }
    }

    let tray_menu = Menu::new();

    let addr_i = MenuItem::new(format!("http://{}/", listen), true, None);
    let help_i = MenuItem::new("Help", true, None);
    let quit_i = MenuItem::new("Exit", true, None);
    tray_menu.append_items(&[&addr_i, &help_i, &PredefinedMenuItem::separator(), &quit_i])?;

    let icon = load_icon();
    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_tooltip("BMS Kneeboard Server")
            .with_menu(Box::new(tray_menu))
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_, event_loop| {
        event_loop.set_control_flow(ControlFlow::Wait);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                tray_icon.take();
                event_loop.exit();
            }
            if event.id == addr_i.id() {
                if let Err(e) = open::that(format!("http://{}", listen)) {
                    eprintln!("Failed to open: {:?}", e);
                }
            }
            if event.id == help_i.id() {
                if let Err(e) = open::that(
                    "https://github.com/aviinl/bms-kneeboard-server#bms-kneeboard-server",
                ) {
                    eprintln!("Failed to open: {:?}", e);
                }
            }
        }
    })?;

    Ok(())
}

fn load_icon() -> tray_icon::Icon {
    let icon = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png"));
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
