mod icon_data;

#[derive(Debug)]
struct MyMenuId {
    test_notification: String,
    exit: String,
}

fn main() {
    if let Err(err) = try_main() {
        dbg!(err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::UI::WindowsAndMessaging;

    let local_ip_address = local_ip_address::local_ip()?;
    let port = get_port()?;
    dbg!(local_ip_address, port);

    let (_tray_icon, my_menu_id) = create_tray_icon(local_ip_address, port)?;

    let server = std::net::TcpListener::bind((local_ip_address, port))?;
    std::thread::spawn(|| handle_server(server));

    let mut message = WindowsAndMessaging::MSG::default();
    let menu_event_receiver = tray_icon::menu::MenuEvent::receiver();
    unsafe {
        while WindowsAndMessaging::GetMessageW(&mut message, None, 0, 0).as_bool() {
            if let Ok(event) = menu_event_receiver.try_recv() {
                match event.id {
                    id if id == my_menu_id.test_notification => {
                        show_notification("Test Notification\nThis is a test message.")
                    }
                    id if id == my_menu_id.exit => WindowsAndMessaging::PostQuitMessage(0),
                    _ => {
                        dbg!("Unknown menu event", event);
                    }
                }
            }
            WindowsAndMessaging::DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn handle_server(server: std::net::TcpListener) {
    use std::io::Read;
    dbg!("Server started");
    for stream in server.incoming() {
        let mut buffer = [0; 1024];
        match stream {
            Ok(mut stream) => {
                // 通知領域に限界があるため、読み込みきれなくても無視する
                let _ = stream
                    .read(&mut buffer)
                    // 途中で切断された場合はエラーとして扱わない
                    .or_else(|err| {
                        if err.kind() == std::io::ErrorKind::UnexpectedEof {
                            Ok(0)
                        } else {
                            Err(err)
                        }
                    })
                    .expect("Failed to read from stream");
            }
            Err(err) => {
                dbg!(err);
                continue;
            }
        }
        let message = std::str::from_utf8(&buffer)
            .expect("Failed to convert buffer to string")
            .trim_end_matches('\0')
            .trim_end();
        show_notification(message);

        dbg!("Connection closed");
    }
    dbg!("Server stopped");
}

fn show_notification(message: &str) {
    use winrt_toast::{Scenario, Toast, ToastManager};

    // applicationid に PowerShell を指定すると、Toastをクリックしてもウィンドウが開かれない
    let manager = ToastManager::new(
        r#"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe"#,
    );
    let mut toast = Toast::new();
    toast.scenario(Scenario::Reminder);

    let mut line_iter = message.lines();
    if let Some(text1) = line_iter.next() {
        toast.text1(text1);
        if let Some(text2) = line_iter.next() {
            toast.text2(text2);
            if let Some(text3) = line_iter.next() {
                toast.text3(text3);
                // 4行目以降は無視する
            }
        }
    }

    manager.show(&toast).expect("Failed to show toast");
}

fn get_port() -> Result<u16, Box<dyn std::error::Error>> {
    use std::env;

    env::args()
        .nth(1)
        .or_else(|| env::var("MY_NOTIF_PORT").ok())
        .map_or_else(|| Ok(45654), |s| s.parse().map_err(Into::into))
}

fn create_tray_icon(
    ip: std::net::IpAddr,
    port: u16,
) -> Result<(tray_icon::TrayIcon, MyMenuId), Box<dyn std::error::Error>> {
    use tray_icon::{
        Icon, TrayIconBuilder,
        menu::{Menu, MenuItem},
    };

    let test_notification = MenuItem::new("Test Notification", true, None);
    let exit = MenuItem::new("Exit", true, None);
    let menu = Menu::with_items(&[&test_notification, &exit])?;

    let menu_id = MyMenuId {
        test_notification: test_notification.id().0.to_owned(),
        exit: exit.id().0.to_owned(),
    };

    let icon = Icon::from_rgba(
        icon_data::ICON_RGBA.to_vec(),
        icon_data::ICON_WIDTH,
        icon_data::ICON_HEIGHT,
    )?;

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip(format!("My Notif\nIP: {ip}\nPort: {port}"))
        .with_menu_on_left_click(true)
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()?;

    Ok((tray_icon, menu_id))
}
