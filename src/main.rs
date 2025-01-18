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
                return Ok(());
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

    TrayIconBuilder::new()
        .with_tooltip("[tooltip]\nTray Icon")
        .with_menu_on_left_click(true)
        .with_menu(Box::new(menu))
        .with_icon(Icon::from_rgba(vec![0xFF, 0xFF, 0x00, 0x00], 1, 1).unwrap())
        .build()
        .map_err(Into::into)
}
