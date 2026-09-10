//! Lekhani Desktop GUI Application

slint::include_modules!();

mod tray;

use std::rc::Rc;
use tracing::info;
use lekhani_core::{bijoy_to_unicode, unicode_to_bijoy, PhoneticDatabase, PhoneticSuggestion};
use lekhani_settings::{ConfigManager, LayoutManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Lekhani GUI Desktop Suite...");

    let app = TopBarWindow::new()?;
    let app_weak = app.as_weak();

    let config_mgr = ConfigManager::new();
    let mut layout_mgr = LayoutManager::new();
    let mut db = PhoneticDatabase::new();

    let system_dir = ConfigManager::get_system_layout_dir();
    let user_dir = config_mgr.get_user_layout_dir();
    layout_mgr.discover_layouts(system_dir, user_dir);

    let system_data = ConfigManager::get_system_data_dir();
    let _ = db.load_from_dir(system_data);
    db.load_user_autocorrect(config_mgr.get_user_autocorrect_path());

    // Initialize UI state
    let current_layout = config_mgr.config.general.active_layout.clone();
    app.set_active_layout_name(current_layout.clone().into());
    let layouts = layout_mgr.get_layout_list();
    let slint_layouts: Vec<slint::SharedString> = layouts.into_iter().map(|s| s.into()).collect();
    let model = std::rc::Rc::new(slint::VecModel::from(slint_layouts));
    app.set_available_layouts(model.into());

    // Settings state
    app.set_set_use_dict(config_mgr.config.phonetic.use_dictionary);
    app.set_set_include_eng(config_mgr.config.phonetic.include_english);
    app.set_set_enter_closes(config_mgr.config.phonetic.enter_key_closes_candidate_window);
    app.set_set_auto_vowel(config_mgr.config.fixed.auto_vowel_forming);
    app.set_set_auto_chandra(config_mgr.config.fixed.auto_chandra_position);
    app.set_set_traditional_kar(config_mgr.config.fixed.traditional_kar);
    app.set_set_old_reph(config_mgr.config.fixed.old_reph);
    app.set_set_numberpad(config_mgr.config.fixed.numberpad);

    // Initialize Desktop System Tray (ksni)
    let _tray_handle = tray::spawn_tray(current_layout);

    // Callbacks: Layout Switching
    let config_mgr_rc = Rc::new(std::cell::RefCell::new(config_mgr));
    let cm_clone = config_mgr_rc.clone();
    let app_weak_layout = app_weak.clone();
    app.on_select_layout(move |name| {
        let mut cm = cm_clone.borrow_mut();
        cm.config.general.active_layout = name.to_string();
        cm.save();
        if let Some(app) = app_weak_layout.upgrade() {
            app.set_active_layout_name(name);
            app.set_show_layout_menu(false);
        }
    });

    // Callbacks: Bijoy Converter
    let app_weak_conv = app_weak.clone();
    app.on_convert_bijoy_to_unicode(move |text| {
        let converted = bijoy_to_unicode(&text);
        if let Some(app) = app_weak_conv.upgrade() {
            app.set_conv_output_text(converted.into());
        }
    });

    let app_weak_conv_rev = app_weak.clone();
    app.on_convert_unicode_to_bijoy(move |text| {
        let converted = unicode_to_bijoy(&text);
        if let Some(app) = app_weak_conv_rev.upgrade() {
            app.set_conv_output_text(converted.into());
        }
    });

    // Callbacks: AutoCorrect live input preview
    let phonetic_sugg = PhoneticSuggestion::new();
    let ps_rc = Rc::new(std::cell::RefCell::new(phonetic_sugg));
    let ps_clone = ps_rc.clone();
    let app_weak_ac = app_weak.clone();
    app.on_ac_input_changed(move |replace_text, with_text| {
        let ps = ps_clone.borrow();
        let prev_r = ps.convert_phonetic(&replace_text);
        let prev_w = ps.convert_phonetic(&with_text);
        if let Some(app) = app_weak_ac.upgrade() {
            app.set_ac_preview_replace(prev_r.into());
            app.set_ac_preview_with(prev_w.into());
        }
    });

    // Callbacks: Settings Save
    let cm_save = config_mgr_rc.clone();
    let app_weak_set = app_weak.clone();
    app.on_save_settings(move || {
        if let Some(app) = app_weak_set.upgrade() {
            let mut cm = cm_save.borrow_mut();
            cm.config.phonetic.use_dictionary = app.get_set_use_dict();
            cm.config.phonetic.include_english = app.get_set_include_eng();
            cm.config.phonetic.enter_key_closes_candidate_window = app.get_set_enter_closes();
            cm.config.fixed.auto_vowel_forming = app.get_set_auto_vowel();
            cm.config.fixed.auto_chandra_position = app.get_set_auto_chandra();
            cm.config.fixed.traditional_kar = app.get_set_traditional_kar();
            cm.config.fixed.old_reph = app.get_set_old_reph();
            cm.config.fixed.numberpad = app.get_set_numberpad();
            cm.save();
            app.set_show_settings_dialog(false);
        }
    });

    // Callbacks: Quit
    app.on_trigger_quit(move || {
        let _ = slint::quit_event_loop();
    });

    app.run()?;
    Ok(())
}
