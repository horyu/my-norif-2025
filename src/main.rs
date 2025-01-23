#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icon_data;

#[derive(Debug)]
struct MyMenuId {
    test_notification: String,
    exit: String,
}

fn main() {
    if let Err(err) = try_main() {
        dbg!(&err);
        handle_error(err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_ip_address = local_ip_address::local_ip()?;
    let port = get_port()?;
    dbg!(local_ip_address, port);

    let (_tray_icon, my_menu_id) = create_tray_icon(local_ip_address, port)?;

    let server = std::net::TcpListener::bind((local_ip_address, port))?;
    let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
    let join_handle = std::thread::spawn(move || {
        let result = handle_server(server);
        post_quit_message_to_thread(thread_id);
        result
    });

    process_message_loop(my_menu_id)?;

    if join_handle.is_finished() {
        join_handle
            .join()
            .map_err(|e| format!("Failed to join thread: {:#?}", e))?
    } else {
        Ok(())
    }
}

fn process_message_loop(
    my_menu_id: MyMenuId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use windows::Win32::UI::WindowsAndMessaging;

    let mut message = WindowsAndMessaging::MSG::default();
    let menu_event_receiver = tray_icon::menu::MenuEvent::receiver();

    unsafe {
        while WindowsAndMessaging::GetMessageW(&mut message, None, 0, 0).as_bool() {
            if let Ok(event) = menu_event_receiver.try_recv() {
                match event.id {
                    id if id == my_menu_id.test_notification => {
                        show_notification("Test Notification\nThis is a test message.")?;
                    }
                    id if id == my_menu_id.exit => WindowsAndMessaging::PostQuitMessage(0),
                    _ => return Err(format!("Unknown menu event: {event:#?}").into()),
                }
            }
            WindowsAndMessaging::DispatchMessageW(&message);
        }
    }
    Ok(())
}

fn handle_server(
    server: std::net::TcpListener,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Read;
    const MAX_NOTIFICATION_LENGTH: usize = 1024;

    for stream in server.incoming() {
        // 通知領域に限界があるため、読み込みきれなくても無視する
        let mut buffer = [0; MAX_NOTIFICATION_LENGTH];
        let _ = stream?.read(&mut buffer).or_else(|err| {
            // 途中で切断された場合はエラーとして扱わない
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                Ok(0)
            } else {
                Err(err)
            }
        })?;
        let message = std::str::from_utf8(&buffer)?
            .trim_end_matches('\0')
            .trim_end();
        show_notification(message)?;
    }

    Err("Server stopped.".into())
}

fn show_notification(message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use winrt_toast::{Scenario, Toast, ToastManager};
    const POWERSHELL_APP_ID: &str =
        r#"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe"#;

    // applicationid に PowerShell を指定すると、Toastをクリックしてもウィンドウが開かれない
    let manager = ToastManager::new(POWERSHELL_APP_ID);
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

    manager.show(&toast).map_err(Into::into)
}

fn post_quit_message_to_thread(thread_id: u32) {
    use windows::Win32::{Foundation, UI::WindowsAndMessaging};

    unsafe {
        let _ = WindowsAndMessaging::PostThreadMessageW(
            thread_id,
            WindowsAndMessaging::WM_QUIT,
            Foundation::WPARAM::default(),
            Foundation::LPARAM::default(),
        );
    }
}

fn get_port() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    use std::env;
    const DEFAULT_PORT: u16 = 45654;

    env::args()
        .nth(1)
        .or_else(|| env::var("MY_NOTIF_PORT").ok())
        .map_or_else(|| Ok(DEFAULT_PORT), |s| s.parse().map_err(Into::into))
}

fn create_tray_icon(
    ip: std::net::IpAddr,
    port: u16,
) -> Result<(tray_icon::TrayIcon, MyMenuId), Box<dyn std::error::Error + Send + Sync>> {
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

fn handle_error(err: Box<dyn std::error::Error + Send + Sync>) {
    use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
    use std::{fs::File, io::Write};

    if MessageDialogResult::Yes
        != MessageDialog::new()
            .set_title("My Notif")
            .set_level(MessageLevel::Error)
            .set_buttons(MessageButtons::YesNo)
            .set_description(
                "An error occurred. Would you like to save the error details to a file?",
            )
            .show()
    {
        return;
    }

    let Some(file_path) = FileDialog::new()
        .set_title("MyNotif Error Log")
        .add_filter("Log File", &["log"])
        .save_file()
    else {
        return;
    };

    if let Err(e) = File::create(file_path).and_then(|mut file| writeln!(file, "{err:#?}")) {
        MessageDialog::new()
            .set_title("My Notif")
            .set_level(MessageLevel::Error)
            .set_description(format!("Failed to save the error details.\n{e:#?}"))
            .show();
    }
}
