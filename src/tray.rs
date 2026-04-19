use crate::UserEvent;
use anyhow::Result;
use image::{GenericImageView, load_from_memory};
use std::error::Error;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::event_loop::EventLoopProxy;

pub struct Tray {
    _tray_icon: TrayIcon,
    menu_item_server: MenuItem,
    menu_item_exit: MenuItem,
}

impl Tray {
    pub fn new() -> Self {
        let icon = load_icon().unwrap();
        let menu_item_server = new_menu_item("服务: 未运行");
        let menu_item_exit = new_menu_item("退出");
        let menu = new_menu(vec![&menu_item_server, &menu_item_exit]).unwrap();

        let tray_icon = new_icon(icon, menu).unwrap();
        tray_icon.set_show_menu_on_left_click(false);

        Self {
            _tray_icon: tray_icon,
            menu_item_server,
            menu_item_exit,
        }
    }

    pub fn start_listening_events(&self, proxy: EventLoopProxy<UserEvent>) {
        let menu_id_exit = self.menu_item_exit.id().clone();
        MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
            if ev.id() == &menu_id_exit {
                let _ = proxy.send_event(UserEvent::Exit);
            };
        }));
    }

    pub fn set_server_text(&mut self, text: String) -> Result<()> {
        self.menu_item_server.set_text(&text);

        Ok(())
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

fn new_menu(menu_items: Vec<&MenuItem>) -> Result<Menu, Box<dyn Error>> {
    let menu = Menu::new();

    for it in menu_items {
        menu.append(it)?;
    }

    Ok(menu)
}

fn new_icon(icon: Icon, menu: Menu) -> Result<TrayIcon, Box<dyn Error>> {
    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_title("Clipclip")
        .with_tooltip("Clipclip")
        .with_menu(Box::new(menu))
        .build()?;

    Ok(tray_icon)
}
