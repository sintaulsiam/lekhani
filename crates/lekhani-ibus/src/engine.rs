//! IBus Engine Implementation with zbus

use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use zbus::interface;

use lekhani_core::{ActiveLayoutType, InputSession, MODIFIER_ALT_GR, MODIFIER_SHIFT, VC_UNKNOWN};
use lekhani_settings::{ConfigManager, LayoutManager};
use crate::keycode::KeycodeMapper;

// Special IBus Key Values
const IBUS_KEY_BACKSPACE: u32 = 0xff08;
const IBUS_KEY_RETURN: u32 = 0xff0d;
const IBUS_KEY_KP_ENTER: u32 = 0xff8d;
const IBUS_KEY_SPACE: u32 = 0x0020;
const IBUS_KEY_LEFT: u32 = 0xff51;
const IBUS_KEY_UP: u32 = 0xff52;
const IBUS_KEY_RIGHT: u32 = 0xff53;
const IBUS_KEY_DOWN: u32 = 0xff54;
const IBUS_KEY_TAB: u32 = 0xff09;
const IBUS_KEY_ALT_R: u32 = 0xffea;
const IBUS_KEY_ISO_LEVEL3_SHIFT: u32 = 0xfe03;
const IBUS_RELEASE_MASK: u32 = 1 << 30;

pub struct IBusEngineState {
    pub session: InputSession,
    pub config_mgr: ConfigManager,
    #[allow(dead_code)]
    pub layout_mgr: LayoutManager,
    pub mapper: KeycodeMapper,
    pub alt_gr: bool,
}

pub struct LekhaniIBusEngine {
    pub state: Arc<Mutex<IBusEngineState>>,
}

impl LekhaniIBusEngine {
    pub fn new() -> Self {
        let config_mgr = ConfigManager::new();
        let mut layout_mgr = LayoutManager::new();
        
        let system_dir = ConfigManager::get_system_layout_dir();
        let user_dir = config_mgr.get_user_layout_dir();
        layout_mgr.discover_layouts(system_dir, user_dir);

        let mut session = InputSession::new();
        let system_data = ConfigManager::get_system_data_dir();
        let user_ac = config_mgr.get_user_autocorrect_path();
        let user_learned = config_mgr.get_user_learned_path();
        let stats_path = config_mgr.get_data_dir().join("stats.json");
        session.load_database(&system_data);
        session.load_user_autocorrect(&user_ac);
        session.load_user_learned(&user_learned);
        session.load_stats(&stats_path);
        
        // Load active layout
        let active_name = &config_mgr.config.general.active_layout;
        if let Some(json) = layout_mgr.load_layout_json(active_name) {
            let layout_type = if layout_mgr.get_layout(active_name).map(|i| i.layout_type.as_str()) == Some("fixed") {
                ActiveLayoutType::Fixed
            } else {
                ActiveLayoutType::Phonetic
            };
            session.set_layout(layout_type, &json);
        }

        let state = IBusEngineState {
            session,
            config_mgr,
            layout_mgr,
            mapper: KeycodeMapper::new(),
            alt_gr: false,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl LekhaniIBusEngine {
    async fn process_key_event(
        &mut self,
        keyval: u32,
        _keycode: u32,
        state_mask: u32,
    ) -> zbus::fdo::Result<bool> {
        let mut st = self.state.lock().unwrap();

        // Check key release
        if (state_mask & IBUS_RELEASE_MASK) != 0 {
            if keyval == IBUS_KEY_ALT_R || keyval == IBUS_KEY_ISO_LEVEL3_SHIFT {
                st.alt_gr = false;
            }
            return Ok(false);
        }

        // Auto-sync configuration and autocorrect if modified externally
        if st.config_mgr.check_and_reload() {
            let user_ac = st.config_mgr.get_user_autocorrect_path();
            st.session.load_user_autocorrect(&user_ac);
        }

        // Special handling for navigation and triggers
        match keyval {
            IBUS_KEY_BACKSPACE => {
                if st.session.is_active() {
                    let handled = st.session.process_backspace();
                    return Ok(handled);
                }
                return Ok(false);
            }
            IBUS_KEY_RETURN => {
                if st.session.is_active() {
                    let idx = st.session.get_selected_index();
                    let _ = st.session.commit(idx);
                    return Ok(st.config_mgr.config.phonetic.enter_key_closes_candidate_window);
                }
                return Ok(false);
            }
            IBUS_KEY_SPACE | IBUS_KEY_KP_ENTER => {
                if st.session.is_active() {
                    let idx = st.session.get_selected_index();
                    let _ = st.session.commit(idx);
                }
                return Ok(false);
            }
            IBUS_KEY_RIGHT | IBUS_KEY_DOWN => {
                if st.session.is_active() {
                    st.session.select_next();
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_LEFT | IBUS_KEY_UP => {
                if st.session.is_active() {
                    st.session.select_prev();
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_TAB => {
                if st.session.is_active() {
                    st.session.select_next();
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_ALT_R | IBUS_KEY_ISO_LEVEL3_SHIFT => {
                st.alt_gr = true;
                return Ok(st.session.is_active());
            }
            _ => {}
        }

        let mut mod_mask = 0u8;
        if (state_mask & (1 << 0)) != 0 { // Shift
            mod_mask |= MODIFIER_SHIFT;
        }
        if st.alt_gr || ((state_mask & (1 << 2)) != 0 && (state_mask & (1 << 3)) != 0) {
            mod_mask |= MODIFIER_ALT_GR;
        }

        let vc = st.mapper.map_keyval(keyval);
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

    async fn focus_in(&mut self) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Focus In");
        Ok(())
    }

    async fn focus_out(&mut self) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Focus Out");
        let mut st = self.state.lock().unwrap();
        st.session.reset();
        Ok(())
    }

    async fn reset(&mut self) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Reset");
        let mut st = self.state.lock().unwrap();
        st.session.reset();
        Ok(())
    }

    async fn enable(&mut self) -> zbus::fdo::Result<()> {
        info!("IBus Engine Enabled");
        let mut st = self.state.lock().unwrap();
        st.config_mgr.load();
        Ok(())
    }

    async fn disable(&mut self) -> zbus::fdo::Result<()> {
        info!("IBus Engine Disabled");
        let mut st = self.state.lock().unwrap();
        let user_learned = st.config_mgr.get_user_learned_path();
        let stats_path = st.config_mgr.get_data_dir().join("stats.json");
        let _ = st.session.save_user_learned(&user_learned);
        let _ = st.session.save_stats(&stats_path);
        st.session.reset();
        Ok(())
    }
}
