mod icon_data;

fn main() {
    if let Err(err) = try_main() {
        dbg!(err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::UI::WindowsAndMessaging;

    let _tray_icon = create_tray_icon()?;

    let local_ip_address = local_ip_address::local_ip()?;
    dbg!(local_ip_address);

    let server = std::net::TcpListener::bind((local_ip_address, 45654))?;
    std::thread::spawn(|| handle_server(server));

    let mut message = WindowsAndMessaging::MSG::default();
    let menu_event_receiver = tray_icon::menu::MenuEvent::receiver();
    unsafe {
        while WindowsAndMessaging::GetMessageW(&mut message, None, 0, 0).as_bool() {
            // Menu は Exit のみであるため、MenuEvent が発生したら終了する
            if menu_event_receiver.try_recv().is_ok() {
                WindowsAndMessaging::PostQuitMessage(0);
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
