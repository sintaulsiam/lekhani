//! Native Linux System Tray Icon (StatusNotifierItem via pure Rust zbus)

use zbus::interface;
use zbus::connection::Builder;

pub struct LekhaniTray {
    pub active_layout: String,
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
        format!("Lekhani ({})", self.active_layout)
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
        ""
    }

    #[zbus(property)]
    fn item_is_menu(&self) -> bool {
        false
    }

    fn activate(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: Activate triggered");
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: SecondaryActivate triggered");
    }

    fn scroll(&self, _delta: i32, _orientation: &str) {
        tracing::info!("Tray: Scroll triggered");
    }

    fn context_menu(&self, _x: i32, _y: i32) {
        tracing::info!("Tray: ContextMenu triggered");
    }
}

pub fn spawn_tray(active_layout: String) -> Option<std::thread::JoinHandle<()>> {
    let handle = std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("Failed to create tokio runtime for tray: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let item = LekhaniTray { active_layout };
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
            let _ = conn.call_method(
                Some("org.kde.StatusNotifierWatcher"),
                "/StatusNotifierWatcher",
                Some("org.kde.StatusNotifierWatcher"),
                "RegisterStatusNotifierItem",
                &service_name,
            ).await;

            tracing::info!("Lekhani StatusNotifierItem tray running on DBus: {}", service_name);
            futures_util::future::pending::<()>().await;
        });
    });

    Some(handle)
}
