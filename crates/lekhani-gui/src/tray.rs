//! Native Linux System Tray Icon (StatusNotifierItem via pure Rust zbus)

use std::sync::{Arc, RwLock};
use zbus::connection::Builder;
use zbus::interface;

pub enum TrayCommand {
    UpdateLayout(String),
}

pub struct LekhaniTray {
    pub active_layout: Arc<RwLock<String>>,
    pub app_weak: slint::Weak<crate::TopBarWindow>,
}

#[interface(name = "org.kde.StatusNotifierItem")]
impl LekhaniTray {
    #[zbus(property)]
    fn category(&self) -> &str {
        "ApplicationStatus"
    }

    #[zbus(property)]
    fn id(&self) -> &str {
        "lekhani-indicator"
    }

    #[zbus(property)]
    fn title(&self) -> String {
        let layout = self.active_layout.read().unwrap();
        format!("Lekhani ({})", *layout)
    }

    #[zbus(property)]
    fn status(&self) -> &str {
        "Active"
    }

    #[zbus(property)]
    fn icon_name(&self) -> &str {
        "lekhani"
    }

    #[zbus(property)]
    fn overlay_icon_name(&self) -> &str {
        ""
    }

    #[zbus(property)]
    fn attention_icon_name(&self) -> &str {
        ""
    }

    #[zbus(property)]
    fn icon_theme_path(&self) -> &str {
        "/usr/share/icons/hicolor"
    }

    #[zbus(property)]
    fn item_is_menu(&self) -> bool {
        false
    }

    fn activate(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: Activate triggered - toggling layout menu");
        let app_weak = self.app_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                let cur = app.get_active_dialog();
                if cur == 0 {
                    app.set_active_dialog(1);
                } else {
                    app.set_active_dialog(0);
                }
            }
        });
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: SecondaryActivate triggered - toggling layout mode");
        let app_weak = self.app_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                app.invoke_toggle_layout_mode();
            }
        });
    }

    fn scroll(&self, delta: i32, _orientation: &str) {
        tracing::info!("Tray: Scroll triggered: {}", delta);
    }

    fn context_menu(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: ContextMenu triggered - toggling layout mode");
        let app_weak = self.app_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                app.invoke_toggle_layout_mode();
            }
        });
    }

    #[zbus(signal)]
    pub async fn new_title(emitter: &zbus::object_server::SignalContext<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn new_icon(emitter: &zbus::object_server::SignalContext<'_>) -> zbus::Result<()>;
}

#[derive(Clone)]
pub struct TrayHandle {
    pub tx: tokio::sync::mpsc::UnboundedSender<TrayCommand>,
}

impl TrayHandle {
    pub fn update_layout(&self, layout: &str) {
        let _ = self.tx.send(TrayCommand::UpdateLayout(layout.to_string()));
    }
}

pub fn spawn_tray(
    active_layout: String,
    app_weak: slint::Weak<crate::TopBarWindow>,
) -> Option<TrayHandle> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TrayCommand>();
    let layout_arc = Arc::new(RwLock::new(active_layout));
    let layout_for_thread = layout_arc.clone();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("Failed to create tokio runtime for tray: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let item = LekhaniTray {
                active_layout: layout_for_thread.clone(),
                app_weak,
            };
            let pid = std::process::id();
            let service_name = format!("org.kde.StatusNotifierItem-{}-1", pid);

            let conn_res = Builder::session();
            let b = match conn_res {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Failed to connect to session DBus for tray: {}", e);
                    return;
                }
            };

            let b = match b.name(service_name.clone()) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Failed to request DBus name for tray: {}", e);
                    return;
                }
            };

            let conn = match b.serve_at("/StatusNotifierItem", item) {
                Ok(b) => match b.build().await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to build DBus connection for tray: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to serve StatusNotifierItem: {}", e);
                    return;
                }
            };

            // Register with StatusNotifierWatcher if available
            let _ = conn
                .call_method(
                    Some("org.kde.StatusNotifierWatcher"),
                    "/StatusNotifierWatcher",
                    Some("org.kde.StatusNotifierWatcher"),
                    "RegisterStatusNotifierItem",
                    &service_name,
                )
                .await;

            tracing::info!(
                "Lekhani StatusNotifierItem tray running on DBus: {}",
                service_name
            );

            // Listen for layout change commands from the GUI
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    TrayCommand::UpdateLayout(new_name) => {
                        {
                            let mut w = layout_for_thread.write().unwrap();
                            *w = new_name;
                        }
                        if let Ok(iface_ref) = conn
                            .object_server()
                            .interface::<_, LekhaniTray>("/StatusNotifierItem")
                            .await
                        {
                            let _ = LekhaniTray::new_title(iface_ref.signal_context()).await;
                        }
                    }
                }
            }
        });
    });

    Some(TrayHandle { tx })
}
