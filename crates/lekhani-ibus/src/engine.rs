//! IBus Engine Implementation with zbus

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use zbus::interface;

use lekhani_core::{
    ActiveLayoutType, InputSession, KeycodeMapper, MODIFIER_ALT_GR, MODIFIER_SHIFT, VC_UNKNOWN,
};
use lekhani_settings::{ConfigManager, LayoutManager};

// Special IBus Key Values
const IBUS_KEY_BACKSPACE: u32 = 0xff08;
const IBUS_KEY_RETURN: u32 = 0xff0d;
const IBUS_KEY_KP_ENTER: u32 = 0xff8d;
const IBUS_KEY_SPACE: u32 = 0x0020;
const IBUS_KEY_ESCAPE: u32 = 0xff1b;
const IBUS_KEY_LEFT: u32 = 0xff51;
const IBUS_KEY_UP: u32 = 0xff52;
const IBUS_KEY_RIGHT: u32 = 0xff53;
const IBUS_KEY_DOWN: u32 = 0xff54;
const IBUS_KEY_TAB: u32 = 0xff09;
const IBUS_KEY_ALT_R: u32 = 0xffea;
const IBUS_KEY_ISO_LEVEL3_SHIFT: u32 = 0xfe03;
const IBUS_KEY_1: u32 = 0x0031;
const IBUS_KEY_5: u32 = 0x0035;
const IBUS_KEY_KP_1: u32 = 0xffb1;
const IBUS_KEY_KP_5: u32 = 0xffb5;
const IBUS_RELEASE_MASK: u32 = 1 << 30;

pub struct IBusEngineState {
    pub session: InputSession,
    pub config_mgr: ConfigManager,
    pub layout_mgr: LayoutManager,
    pub mapper: KeycodeMapper,
    pub alt_gr: bool,
    pub active_layout_name: String,
    pub commit_count: usize,
}

impl IBusEngineState {
    pub fn commit(&mut self, idx: usize) -> Option<String> {
        let committed = self.session.commit(idx)?;
        self.commit_count += 1;
        if self.commit_count.is_multiple_of(60) {
            let is_dirty = self
                .session
                .phonetic
                .suggestion_engine
                .database
                .learner
                .read()
                .map(|l| l.dirty)
                .unwrap_or(false);
            if is_dirty {
                let user_learned = self.config_mgr.get_user_learned_path();
                let stats_path = self.config_mgr.get_user_stats_path();
                let stats_clone = self.session.get_stats();
                let learner_arc = self.session.phonetic.suggestion_engine.database.learner.clone();
                std::thread::spawn(move || {
                    let _ = stats_clone.save_to_path(&stats_path);
                    if let Ok(mut l) = learner_arc.write() {
                        let _ = l.save_to_path(&user_learned);
                    }
                });
            }
        }
        Some(committed)
    }
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
        let stats_path = config_mgr.get_user_stats_path();
        session.load_database(&system_data);
        session.load_user_autocorrect(&user_ac);
        session.load_user_learned(&user_learned);
        session.load_stats(&stats_path);
        config_mgr.config.apply_to_session(&mut session);

        // Load active layout
        let active_name = config_mgr.config.general.active_layout.clone();
        if let Some(json) = layout_mgr.load_layout_json(&active_name) {
            let layout_type = if layout_mgr
                .get_layout(&active_name)
                .map(|i| i.layout_type.as_str())
                == Some("fixed")
            {
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
            active_layout_name: active_name,
            commit_count: 0,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl LekhaniIBusEngine {
    #[zbus(signal)]
    async fn commit_text(
        emitter: &zbus::object_server::SignalContext<'_>,
        text: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        emitter: &zbus::object_server::SignalContext<'_>,
        text: &str,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_preedit_text(
        emitter: &zbus::object_server::SignalContext<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_lookup_table(
        emitter: &zbus::object_server::SignalContext<'_>,
        candidates: &[String],
        selected_index: u32,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_lookup_table(
        emitter: &zbus::object_server::SignalContext<'_>,
    ) -> zbus::Result<()>;

    async fn process_key_event(
        &mut self,
        #[zbus(signal_context)] emitter: zbus::object_server::SignalContext<'_>,
        keyval: u32,
        _keycode: u32,
        state_mask: u32,
    ) -> zbus::fdo::Result<bool> {
        let mut st = self.state.lock().await;

        // Check key release
        if (state_mask & IBUS_RELEASE_MASK) != 0 {
            if keyval == IBUS_KEY_ALT_R || keyval == IBUS_KEY_ISO_LEVEL3_SHIFT {
                st.alt_gr = false;
            }
            return Ok(false);
        }

        // Standalone modifier keys (Shift_L, Shift_R, Ctrl, Alt, Super, AltGr, CapsLock)
        // must NOT commit active preedit or dismiss the candidate window.
        if matches!(keyval, 0xffe1..=0xffee | 0xfe03 | 0xfe08 | 0xfe09) {
            if keyval == IBUS_KEY_ALT_R || keyval == IBUS_KEY_ISO_LEVEL3_SHIFT {
                st.alt_gr = true;
            }
            return Ok(false);
        }

        // Auto-sync configuration, autocorrect, and layout if modified externally
        if st.config_mgr.check_and_reload() {
            let cfg = st.config_mgr.config.clone();
            cfg.apply_to_session(&mut st.session);
            let user_ac = st.config_mgr.get_user_autocorrect_path();
            st.session.load_user_autocorrect(&user_ac);
            let user_learned = st.config_mgr.get_user_learned_path();
            if user_learned.exists() {
                st.session.load_user_learned(&user_learned);
            }

            let new_layout = st.config_mgr.config.general.active_layout.clone();
            if new_layout != st.active_layout_name {
                let system_dir = ConfigManager::get_system_layout_dir();
                let user_dir = st.config_mgr.get_user_layout_dir();
                st.layout_mgr.discover_layouts(system_dir, user_dir);

                if let Some(json) = st.layout_mgr.load_layout_json(&new_layout) {
                    let layout_type = if st
                        .layout_mgr
                        .get_layout(&new_layout)
                        .map(|i| i.layout_type.as_str())
                        == Some("fixed")
                    {
                        ActiveLayoutType::Fixed
                    } else {
                        ActiveLayoutType::Phonetic
                    };
                    st.session.set_layout(layout_type, &json);
                    st.active_layout_name = new_layout;
                    st.session.reset();
                }
            }
        }

        // Direct Selection via 1..5 in Prediction Mode
        if st.session.is_prediction_mode() {
            let cand_idx = if (IBUS_KEY_1..=IBUS_KEY_5).contains(&keyval) {
                Some((keyval - IBUS_KEY_1) as usize)
            } else if (IBUS_KEY_KP_1..=IBUS_KEY_KP_5).contains(&keyval) {
                Some((keyval - IBUS_KEY_KP_1) as usize)
            } else {
                None
            };

            if let Some(idx) = cand_idx {
                if idx < st.session.get_candidates().len() {
                    if let Some(committed) = st.commit(idx) {
                        let _ = Self::commit_text(&emitter, &committed).await;
                        if st.config_mgr.config.phonetic.enable_predictive_next_words {
                            st.session.populate_predictions();
                            let cands = st.session.get_candidates();
                            let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                        } else {
                            let _ = Self::hide_lookup_table(&emitter).await;
                            let _ = Self::hide_preedit_text(&emitter).await;
                        }
                        return Ok(true);
                    }
                }
            }
        }

        // Special handling for navigation and triggers
        match keyval {
            IBUS_KEY_BACKSPACE => {
                if st.session.is_active() {
                    let handled = st.session.process_backspace();
                    if st.session.is_active() {
                        let preedit = st.session.get_preedit_text();
                        let cands = st.session.get_candidates();
                        let sel = st.session.get_selected_index() as u32;
                        let _ = Self::update_preedit_text(
                            &emitter,
                            &preedit,
                            preedit.chars().count() as u32,
                            true,
                        )
                        .await;
                        let _ = Self::update_lookup_table(&emitter, cands, sel, true).await;
                    } else {
                        let _ = Self::hide_preedit_text(&emitter).await;
                        let _ = Self::hide_lookup_table(&emitter).await;
                    }
                    return Ok(handled);
                }
                return Ok(false);
            }
            IBUS_KEY_ESCAPE => {
                if st.session.is_active() || st.session.is_prediction_mode() {
                    st.session.reset();
                    let _ = Self::hide_preedit_text(&emitter).await;
                    let _ = Self::hide_lookup_table(&emitter).await;
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_RETURN => {
                if st.session.is_prediction_mode() {
                    if st.session.is_prediction_navigated() {
                        let idx = st.session.get_selected_index();
                        if let Some(committed) = st.commit(idx) {
                            let _ = Self::commit_text(&emitter, &committed).await;
                            if st.config_mgr.config.phonetic.enable_predictive_next_words {
                                st.session.populate_predictions();
                                let cands = st.session.get_candidates();
                                let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                            } else {
                                let _ = Self::hide_lookup_table(&emitter).await;
                                let _ = Self::hide_preedit_text(&emitter).await;
                            }
                        }
                        return Ok(true);
                    } else {
                        st.session.reset();
                        let _ = Self::hide_preedit_text(&emitter).await;
                        let _ = Self::hide_lookup_table(&emitter).await;
                        return Ok(false);
                    }
                }
                if st.session.is_active() {
                    let idx = st.session.get_selected_index();
                    if let Some(committed) = st.commit(idx) {
                        let _ = Self::commit_text(&emitter, &committed).await;
                        if st.config_mgr.config.phonetic.enable_predictive_next_words {
                            st.session.populate_predictions();
                            let cands = st.session.get_candidates();
                            let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                        } else {
                            let _ = Self::hide_lookup_table(&emitter).await;
                            let _ = Self::hide_preedit_text(&emitter).await;
                        }
                    }
                    return Ok(st
                        .config_mgr
                        .config
                        .phonetic
                        .enter_key_closes_candidate_window);
                }
                return Ok(false);
            }
            IBUS_KEY_SPACE | IBUS_KEY_KP_ENTER => {
                if st.session.is_prediction_mode() {
                    if st.session.is_prediction_navigated() {
                        let idx = st.session.get_selected_index();
                        if let Some(committed) = st.commit(idx) {
                            let _ = Self::commit_text(&emitter, &committed).await;
                            if st.config_mgr.config.phonetic.enable_predictive_next_words {
                                st.session.populate_predictions();
                                let cands = st.session.get_candidates();
                                let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                            } else {
                                let _ = Self::hide_lookup_table(&emitter).await;
                                let _ = Self::hide_preedit_text(&emitter).await;
                            }
                        }
                        return Ok(true);
                    } else {
                        st.session.reset();
                        let _ = Self::hide_preedit_text(&emitter).await;
                        let _ = Self::hide_lookup_table(&emitter).await;
                        return Ok(false);
                    }
                }
                if st.session.is_active() {
                    let idx = st.session.get_selected_index();
                    if let Some(committed) = st.commit(idx) {
                        let _ = Self::commit_text(&emitter, &committed).await;
                        if st.config_mgr.config.phonetic.enable_predictive_next_words {
                            st.session.populate_predictions();
                            let cands = st.session.get_candidates();
                            let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                        } else {
                            let _ = Self::hide_lookup_table(&emitter).await;
                            let _ = Self::hide_preedit_text(&emitter).await;
                        }
                    }
                }
                return Ok(false);
            }
            IBUS_KEY_RIGHT | IBUS_KEY_DOWN => {
                if st.session.is_active() || st.session.is_prediction_mode() {
                    st.session.select_next();
                    let cands = st.session.get_candidates();
                    let sel = st.session.get_selected_index() as u32;
                    let _ = Self::update_lookup_table(&emitter, cands, sel, true).await;
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_LEFT | IBUS_KEY_UP => {
                if st.session.is_active() || st.session.is_prediction_mode() {
                    st.session.select_prev();
                    let cands = st.session.get_candidates();
                    let sel = st.session.get_selected_index() as u32;
                    let _ = Self::update_lookup_table(&emitter, cands, sel, true).await;
                    return Ok(true);
                }
                return Ok(false);
            }
            IBUS_KEY_TAB => {
                if st.session.is_active() || st.session.is_prediction_mode() {
                    st.session.select_next();
                    let cands = st.session.get_candidates();
                    let sel = st.session.get_selected_index() as u32;
                    let _ = Self::update_lookup_table(&emitter, cands, sel, true).await;
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

        // Pass modifier hotkeys (Ctrl+C, Ctrl+V, Alt+Tab, etc.) through to app
        let is_ctrl = (state_mask & (1 << 2)) != 0;
        let is_alt = (state_mask & (1 << 3)) != 0;
        if is_ctrl || (is_alt && !st.alt_gr) {
            if st.session.is_active() {
                let idx = st.session.get_selected_index();
                if let Some(committed) = st.commit(idx) {
                    let _ = Self::commit_text(&emitter, &committed).await;
                }
                let _ = Self::hide_preedit_text(&emitter).await;
                let _ = Self::hide_lookup_table(&emitter).await;
            }
            return Ok(false);
        }

        let mut mod_mask = 0u8;
        if (state_mask & (1 << 0)) != 0 {
            // Shift
            mod_mask |= MODIFIER_SHIFT;
        }
        if st.alt_gr || ((state_mask & (1 << 2)) != 0 && (state_mask & (1 << 3)) != 0) {
            mod_mask |= MODIFIER_ALT_GR;
        }

        let vc = st.mapper.map_keyval(keyval);
        if vc == VC_UNKNOWN {
            if st.session.is_active() {
                let idx = st.session.get_selected_index();
                if let Some(committed) = st.commit(idx) {
                    let _ = Self::commit_text(&emitter, &committed).await;
                    if st.config_mgr.config.phonetic.enable_predictive_next_words {
                        st.session.populate_predictions();
                        let cands = st.session.get_candidates();
                        let _ = Self::update_lookup_table(&emitter, cands, 0, true).await;
                    } else {
                        let _ = Self::hide_lookup_table(&emitter).await;
                        let _ = Self::hide_preedit_text(&emitter).await;
                    }
                }
            }
            return Ok(false);
        }

        let handled = st.session.process_key(vc, mod_mask);

        // Update preedit and lookup table signals
        if st.session.is_active() {
            let preedit = st.session.get_preedit_text();
            let cands = st.session.get_candidates();
            let sel = st.session.get_selected_index() as u32;
            let _ =
                Self::update_preedit_text(&emitter, &preedit, preedit.chars().count() as u32, true)
                    .await;
            if !cands.is_empty() {
                let _ = Self::update_lookup_table(&emitter, cands, sel, true).await;
            } else {
                let _ = Self::hide_lookup_table(&emitter).await;
            }
        } else {
            let _ = Self::hide_preedit_text(&emitter).await;
            let _ = Self::hide_lookup_table(&emitter).await;
        }

        Ok(handled)
    }

    async fn focus_in(&mut self) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Focus In");
        Ok(())
    }

    async fn focus_out(
        &mut self,
        #[zbus(signal_context)] emitter: zbus::object_server::SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Focus Out");
        let mut st = self.state.lock().await;
        st.session.reset();
        let _ = Self::hide_preedit_text(&emitter).await;
        let _ = Self::hide_lookup_table(&emitter).await;
        Ok(())
    }

    async fn reset(
        &mut self,
        #[zbus(signal_context)] emitter: zbus::object_server::SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        debug!("IBus Engine Reset");
        let mut st = self.state.lock().await;
        st.session.reset();
        let _ = Self::hide_preedit_text(&emitter).await;
        let _ = Self::hide_lookup_table(&emitter).await;
        Ok(())
    }

    async fn enable(&mut self) -> zbus::fdo::Result<()> {
        info!("IBus Engine Enabled");
        let mut st = self.state.lock().await;
        st.config_mgr.load();
        Ok(())
    }

    async fn disable(
        &mut self,
        #[zbus(signal_context)] emitter: zbus::object_server::SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        info!("IBus Engine Disabled");
        let mut st = self.state.lock().await;
        let user_learned = st.config_mgr.get_user_learned_path();
        let stats_path = st.config_mgr.get_user_stats_path();
        let _ = st.session.save_user_learned(&user_learned);
        let _ = st.session.save_stats(&stats_path);
        st.session.reset();
        let _ = Self::hide_preedit_text(&emitter).await;
        let _ = Self::hide_lookup_table(&emitter).await;
        Ok(())
    }
}
