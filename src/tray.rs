use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use std::sync::mpsc;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Commands the tray can send to the rest of the app.
#[derive(Debug, Clone)]
pub enum TrayCommand {
    Show,
    Quit,
}

/// Holds the tray icon alive for the process lifetime.
pub struct Tray {
    _icon: TrayIcon,
}

impl Tray {
    /// Builds the system-tray icon and context menu.
    ///
    /// Returns the `Tray` handle (must be kept alive) and a receiver for
    /// [`TrayCommand`]s produced by user interaction.
    pub fn new() -> anyhow::Result<(Self, mpsc::Receiver<TrayCommand>)> {
        let (tx, rx) = mpsc::channel::<TrayCommand>();

        let menu = Menu::new();
        let show_item = MenuItem::new("Show", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        menu.append_items(&[&show_item, &PredefinedMenuItem::separator(), &quit_item])?;

        let icon = load_icon();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Chauve")
            .with_icon(icon)
            .build()?;

        // Spawn a thread that forwards tray + menu events as TrayCommands.
        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();
        std::thread::spawn(move || {
            let tray_rx = TrayIconEvent::receiver();
            let menu_rx = MenuEvent::receiver();
            loop {
                // Left-click on the tray icon → show window.
                if let Ok(tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                }) = tray_rx.try_recv()
                {
                    let _ = tx.send(TrayCommand::Show);
                }

                if let Ok(event) = menu_rx.try_recv() {
                    if event.id() == &show_id {
                        let _ = tx.send(TrayCommand::Show);
                    } else if event.id() == &quit_id {
                        let _ = tx.send(TrayCommand::Quit);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        Ok((Self { _icon: tray }, rx))
    }
}

fn load_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("../assets/icons/icon.png");
    let img = image::load_from_memory(bytes)
        .expect("valid icon PNG")
        .into_rgba8();
    let (width, height) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), width, height).expect("valid icon")
}
