use eframe::egui::{Context, ViewportCommand};
use image::{GenericImageView, load_from_memory};
use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender, channel},
};
use tray_icon::{
    Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

pub struct Tray {
    _tray_icon: TrayIcon,
    pub on_exiting: Receiver<()>,
}

impl Tray {
    pub fn new(ctx: Context) -> Self {
        let icon = load_icon().unwrap();
        let on_exit_menu_item = new_menu_item("退出");
        let on_exit_id = on_exit_menu_item.id().clone();
        let menu = new_menu(&vec![on_exit_menu_item]).unwrap();

        let _tray_icon = new_icon(icon, menu).unwrap();
        _tray_icon.set_show_menu_on_left_click(false);

        let (exiting, on_exiting) = channel();

        let event_ctx = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |ev| match ev {
            TrayIconEvent::DoubleClick { button, .. } => {
                if button == MouseButton::Left {
                    handle_double_click(&event_ctx);
                }
            }
            _ => {}
        }));

        let event_ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
            if ev.id() == &on_exit_id {
                handle_exit(&event_ctx, exiting.clone());
            };
        }));

        Self {
            _tray_icon,
            on_exiting,
        }
    }
}

fn load_icon() -> Result<Icon, Box<dyn Error>> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon_dyn_image = load_from_memory(icon_bytes)?;
    let rgba = icon_dyn_image.to_rgba8().to_vec();
    let (width, height) = icon_dyn_image.dimensions();
    let icon = Icon::from_rgba(rgba, width, height)?;

    Ok(icon)
}

fn new_menu_item(text: &str) -> MenuItem {
    MenuItem::new(text, true, None)
}

fn new_menu(menu_items: &Vec<MenuItem>) -> Result<Menu, Box<dyn Error>> {
    let menu = Menu::new();

    for it in menu_items {
        menu.append(it)?;
    }

    Ok(menu)
}

fn new_icon(icon: Icon, menu: Menu) -> Result<TrayIcon, Box<dyn Error>> {
    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()?;

    Ok(tray_icon)
}

fn handle_double_click(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
    if ctx.input(|i| i.viewport().minimized.unwrap_or(false)) {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
    }
    if !ctx.input(|i| i.viewport().focused.unwrap_or(false)) {
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
}

fn handle_exit(ctx: &Context, exiting: Sender<()>) {
    let _ = exiting.send(());
    ctx.send_viewport_cmd(ViewportCommand::Close);
}
