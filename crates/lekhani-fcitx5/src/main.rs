//! Lekhani Fcitx5 Daemon for KDE Plasma & Modern Wayland

use std::sync::{Arc, Mutex};
use tracing::info;
use zbus::interface;

use lekhani_core::{ActiveLayoutType, InputSession, MODIFIER_ALT_GR, MODIFIER_SHIFT, VC_UNKNOWN};
use lekhani_settings::{ConfigManager, LayoutManager};

struct Fcitx5EngineState {
    session: InputSession,
    config_mgr: ConfigManager,
    #[allow(dead_code)]
    layout_mgr: LayoutManager,
    alt_gr: bool,
}

pub struct LekhaniFcitx5Engine {
    state: Arc<Mutex<Fcitx5EngineState>>,
}

impl LekhaniFcitx5Engine {
    pub fn new() -> Self {
        let config_mgr = ConfigManager::new();
        let mut layout_mgr = LayoutManager::new();
        
        let system_dir = ConfigManager::get_system_layout_dir();
        let user_dir = config_mgr.get_user_layout_dir();
        layout_mgr.discover_layouts(system_dir, user_dir);

        let mut session = InputSession::new();
        let active_name = &config_mgr.config.general.active_layout;
        if let Some(json) = layout_mgr.load_layout_json(active_name) {
            let layout_type = if layout_mgr.get_layout(active_name).map(|i| i.layout_type.as_str()) == Some("fixed") {
                ActiveLayoutType::Fixed
            } else {
                ActiveLayoutType::Phonetic
            };
            session.set_layout(layout_type, &json);
        }

        let state = Fcitx5EngineState {
            session,
            config_mgr,
            layout_mgr,
            alt_gr: false,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

#[interface(name = "org.fcitx.Fcitx.InputMethod1")]
impl LekhaniFcitx5Engine {
    async fn process_key_event(
        &mut self,
        keyval: u32,
        _keycode: u32,
        state_mask: u32,
        is_release: bool,
    ) -> zbus::fdo::Result<bool> {
        let mut st = self.state.lock().unwrap();

        if is_release {
            if keyval == 0xff7e || keyval == 0xfe03 {
                st.alt_gr = false;
            }
            return Ok(false);
        }

        // Backspace
        if keyval == 0xff08 {
            if st.session.is_active() {
                return Ok(st.session.process_backspace());
            }
            return Ok(false);
        }

        // Return
        if keyval == 0xff0d {
            if st.session.is_active() {
                let idx = st.session.get_selected_index();
                let _ = st.session.commit(idx);
                return Ok(st.config_mgr.config.phonetic.enter_key_closes_candidate_window);
            }
            return Ok(false);
        }

        // Space
        if keyval == 0x0020 {
            if st.session.is_active() {
                let idx = st.session.get_selected_index();
                let _ = st.session.commit(idx);
            }
            return Ok(false);
        }

        // Arrow selection
        if keyval == 0xff53 || keyval == 0xff54 || keyval == 0xff09 { // Right, Down, Tab
            if st.session.is_active() {
                st.session.select_next();
                return Ok(true);
            }
            return Ok(false);
        }
        if keyval == 0xff51 || keyval == 0xff52 { // Left, Up
            if st.session.is_active() {
                st.session.select_prev();
                return Ok(true);
            }
            return Ok(false);
        }

        let mut mod_mask = 0u8;
        if (state_mask & (1 << 0)) != 0 {
            mod_mask |= MODIFIER_SHIFT;
        }
        if st.alt_gr || ((state_mask & (1 << 2)) != 0 && (state_mask & (1 << 3)) != 0) {
            mod_mask |= MODIFIER_ALT_GR;
        }

        let vc = map_fcitx5_keyval(keyval);
        if vc == VC_UNKNOWN {
            if st.session.is_active() {
                let idx = st.session.get_selected_index();
                let _ = st.session.commit(idx);
            }
            return Ok(false);
        }

        let handled = st.session.process_key(vc, mod_mask);
        Ok(handled)
    }

    async fn reset(&mut self) -> zbus::fdo::Result<()> {
        let mut st = self.state.lock().unwrap();
        st.session.reset();
        Ok(())
    }
}

fn map_fcitx5_keyval(keyval: u32) -> u16 {
    // Standard X11 keyval to Lekhani Virtual Key Code
    match keyval {
        0x0061..=0x007a => keyval as u16 + 41013, // a-z
        0x0041..=0x005a => keyval as u16 + 41075, // A-Z
        0x0030 => 11, // 0
        0x0031..=0x0039 => (keyval - 0x0030 + 1) as u16, // 1-9
        0x002d => 12, // -
        0x003d => 13, // =
        0x005b => 26, // [
        0x005d => 27, // ]
        0x005c => 43, // \
        0x003b => 39, // ;
        0x0027 => 40, // '
        0x002c => 51, // ,
        0x002e => 52, // .
        0x002f => 53, // /
        0x0060 => 41, // `
        _ => VC_UNKNOWN,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Lekhani Fcitx5 Engine Daemon (KDE / Wayland)...");

    let engine = LekhaniFcitx5Engine::new();

    let _conn = zbus::connection::Builder::session()?
        .name("org.fcitx.Fcitx5.Lekhani")?
        .serve_at("/org/fcitx/Fcitx5/Engine", engine)?
        .build()
        .await?;

    info!("Lekhani Fcitx5 engine registered on DBus");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Lekhani Fcitx5 engine...");

    Ok(())
}
