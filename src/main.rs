mod icon_data;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Error: {err:#?}",);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::UI::WindowsAndMessaging;

    let _tray_icon = create_tray_icon()?;

    let menu_event_receiver = tray_icon::menu::MenuEvent::receiver();
    let mut message = WindowsAndMessaging::MSG::default();
    unsafe {
        while WindowsAndMessaging::GetMessageW(&mut message, None, 0, 0).as_bool() {
            // Menu は1アイテムのみであるため、選択イベントが発生したら終了する
            if menu_event_receiver.try_recv().is_ok() {
                WindowsAndMessaging::PostQuitMessage(0);
            }
            WindowsAndMessaging::DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn create_tray_icon() -> Result<tray_icon::TrayIcon, Box<dyn std::error::Error>> {
    use tray_icon::{
        Icon, TrayIconBuilder,
        menu::{Menu, MenuItem},
    };

    let menu = Menu::with_items(&[&MenuItem::new("Exit", true, None)])?;
    let icon = Icon::from_rgba(
        icon_data::ICON_RGBA.to_vec(),
        icon_data::ICON_WIDTH,
        icon_data::ICON_HEIGHT,
    )?;

    TrayIconBuilder::new()
        .with_tooltip("[tooltip]\nTray Icon")
        .with_menu_on_left_click(true)
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()
        .map_err(Into::into)
}
