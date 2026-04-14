use image::{GenericImageView, load_from_memory};
use std::error::Error;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, menu::Menu};

pub struct Tray {
    tray_icon: TrayIcon,
}

impl Tray {
    pub fn new() -> Self {
        let icon = load_icon().unwrap();
        let tray_icon = new_icon(icon).unwrap();

        Self { tray_icon }
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

fn new_menu() -> Result<Menu, Box<dyn Error>> {
    todo!()
}

fn new_icon(icon: Icon) -> Result<TrayIcon, Box<dyn Error>> {
    let tray_icon = TrayIconBuilder::new().with_icon(icon).build()?;

    Ok(tray_icon)
}
