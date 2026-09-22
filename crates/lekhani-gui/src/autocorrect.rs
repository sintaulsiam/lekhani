//! Lekhani AutoCorrect UI Callbacks & Handlers

use crate::{StandaloneDialogWindow, TopBarWindow};
use lekhani_core::{PhoneticDatabase, PhoneticSuggestion};
use lekhani_settings::ConfigManager;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_autocorrect_callbacks(
    app: &TopBarWindow,
    standalone: &StandaloneDialogWindow,
    db_rc: &Rc<RefCell<PhoneticDatabase>>,
    config_mgr_rc: &Rc<RefCell<ConfigManager>>,
) {
    let app_weak = app.as_weak();
    let standalone_weak = standalone.as_weak();

    let phonetic_sugg = PhoneticSuggestion::new();
    let ps_rc = Rc::new(RefCell::new(phonetic_sugg));

    // Callbacks: AutoCorrect live input preview (TopBar)
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

    // Callbacks: AutoCorrect Add / Update / Delete (TopBar)
    let db_add_clone = db_rc.clone();
    let cm_ac_clone = config_mgr_rc.clone();
    let app_weak_add = app_weak.clone();
    let s_weak_add = standalone_weak.clone();
    app.on_ac_add_or_update(move |trigger, replacement| {
        if trigger.is_empty() || replacement.is_empty() {
            return;
        }
        let mut db = db_add_clone.borrow_mut();
        db.insert_user_autocorrect(trigger.to_string(), replacement.to_string());
        let cm = cm_ac_clone.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
        let status = format!(
            "Saved! Total entries: {} (User: {})",
            total,
            db.get_user_autocorrect().len()
        );
        if let Some(app) = app_weak_add.upgrade() {
            app.set_ac_status_text(status.clone().into());
        }
        if let Some(s) = s_weak_add.upgrade() {
            s.set_ac_status_text(status.into());
        }
    });

    let db_del_clone = db_rc.clone();
    let cm_del_clone = config_mgr_rc.clone();
    let app_weak_del = app_weak.clone();
    let s_weak_del = standalone_weak.clone();
    app.on_ac_delete_entry(move |trigger| {
        let mut db = db_del_clone.borrow_mut();
        db.remove_user_autocorrect(&trigger);
        let cm = cm_del_clone.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
        let status = format!(
            "Removed. Total entries: {} (User: {})",
            total,
            db.get_user_autocorrect().len()
        );
        if let Some(app) = app_weak_del.upgrade() {
            app.set_ac_status_text(status.clone().into());
        }
        if let Some(s) = s_weak_del.upgrade() {
            s.set_ac_status_text(status.into());
        }
    });

    let db_search_clone = db_rc.clone();
    let app_weak_search = app_weak.clone();
    app.on_ac_search_changed(move |query| {
        let db = db_search_clone.borrow();
        let user_matches = db
            .get_user_autocorrect()
            .iter()
            .filter(|(k, v)| k.contains(query.as_str()) || v.contains(query.as_str()))
            .count();
        let sys_matches = db
            .get_system_autocorrect()
            .iter()
            .filter(|(k, v)| k.contains(query.as_str()) || v.contains(query.as_str()))
            .count();
        if let Some(app) = app_weak_search.upgrade() {
            app.set_ac_status_text(
                format!(
                    "Found {} matching entries (User: {}, System: {})",
                    user_matches + sys_matches,
                    user_matches,
                    sys_matches
                )
                .into(),
            );
        }
    });

    // Callbacks: Standalone AutoCorrect Search / Add / Delete
    let s_weak_ac_search = standalone_weak.clone();
    let db_search_clone2 = db_rc.clone();
    standalone.on_ac_search_changed(move |query| {
        let db = db_search_clone2.borrow();
        let user_matches = db
            .get_user_autocorrect()
            .iter()
            .filter(|(k, v)| k.contains(query.as_str()) || v.contains(query.as_str()))
            .count();
        let sys_matches = db
            .get_system_autocorrect()
            .iter()
            .filter(|(k, v)| k.contains(query.as_str()) || v.contains(query.as_str()))
            .count();
        if let Some(s) = s_weak_ac_search.upgrade() {
            s.set_ac_status_text(
                format!(
                    "Found {} matching entries (User: {}, System: {})",
                    user_matches + sys_matches,
                    user_matches,
                    sys_matches
                )
                .into(),
            );
        }
    });

    let db_add_clone2 = db_rc.clone();
    let cm_ac_clone2 = config_mgr_rc.clone();
    let s_weak_add2 = standalone_weak.clone();
    let app_weak_add2 = app_weak.clone();
    standalone.on_ac_add_or_update(move |trigger, replacement| {
        if trigger.is_empty() || replacement.is_empty() {
            return;
        }
        let mut db = db_add_clone2.borrow_mut();
        db.insert_user_autocorrect(trigger.to_string(), replacement.to_string());
        let cm = cm_ac_clone2.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
        let status = format!(
            "Saved! Total entries: {} (User: {})",
            total,
            db.get_user_autocorrect().len()
        );
        if let Some(s) = s_weak_add2.upgrade() {
            s.set_ac_status_text(status.clone().into());
        }
        if let Some(app) = app_weak_add2.upgrade() {
            app.set_ac_status_text(status.into());
        }
    });

    let db_del_clone2 = db_rc.clone();
    let cm_del_clone2 = config_mgr_rc.clone();
    let s_weak_del2 = standalone_weak.clone();
    let app_weak_del2 = app_weak.clone();
    standalone.on_ac_delete_entry(move |trigger| {
        let mut db = db_del_clone2.borrow_mut();
        db.remove_user_autocorrect(&trigger);
        let cm = cm_del_clone2.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
        let status = format!(
            "Removed. Total entries: {} (User: {})",
            total,
            db.get_user_autocorrect().len()
        );
        if let Some(s) = s_weak_del2.upgrade() {
            s.set_ac_status_text(status.clone().into());
        }
        if let Some(app) = app_weak_del2.upgrade() {
            app.set_ac_status_text(status.into());
        }
    });
}
