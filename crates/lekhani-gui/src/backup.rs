//! Lekhani Data Backup, Export, Import & Reset Helpers

use crate::settings_sync::{apply_settings_to_app, apply_settings_to_standalone};
use crate::stats::{refresh_standalone_stats_ui, refresh_stats_ui};
use crate::{StandaloneDialogWindow, TopBarWindow};
use lekhani_core::PhoneticDatabase;
use lekhani_settings::ConfigManager;
use std::cell::RefCell;
use std::rc::Rc;

pub fn perform_clear_learned_data(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    let cm = cm_rc.borrow();
    let p = cm.get_user_learned_path();
    let mut learner = lekhani_core::AutonomousLearner::load_from_path(&p);
    learner.clear_user_data();
    let _ = learner.save_to_path(&p);
    let stats_str = format!(
        "{} learned words • {} phrases",
        learner.learned_words.len(),
        learner.user_bigrams.len()
    );
    if let Some(app) = app_opt {
        app.set_learned_stats_text(stats_str.clone().into());
        app.set_settings_status_text("✓ Learned data reset to baseline".into());
    }
    if let Some(s) = s_opt {
        s.set_learned_stats_text(stats_str.into());
        s.set_settings_status_text("✓ Learned data reset to baseline".into());
    }
}

pub fn perform_restore_defaults(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    let mut cm = cm_rc.borrow_mut();
    let active_layout = cm.config.general.active_layout.clone();
    let mut def = lekhani_settings::AppConfig::default();
    def.general.active_layout = active_layout;
    cm.config = def;
    cm.save();
    if let Some(app) = app_opt {
        apply_settings_to_app(app, &cm.config);
        app.set_settings_status_text("✓ Defaults restored!".into());
    }
    if let Some(s) = s_opt {
        apply_settings_to_standalone(s, &cm.config);
        s.set_settings_status_text("✓ Defaults restored!".into());
    }
}

pub fn perform_export_full_backup(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    if let Some(dest) = rfd::FileDialog::new()
        .set_title("Export Full Lekhani Backup")
        .add_filter("Lekhani Backup (*.json)", &["json"])
        .set_file_name("lekhani_full_backup.json")
        .save_file()
    {
        let cm = cm_rc.borrow();
        let msg = match cm.export_backup(&dest) {
            Ok(_) => "✓ Backup bundle exported successfully!".to_string(),
            Err(e) => format!("Export failed: {}", e),
        };
        if let Some(app) = app_opt {
            app.set_settings_status_text(msg.clone().into());
        }
        if let Some(s) = s_opt {
            s.set_settings_status_text(msg.into());
        }
    }
}

pub fn perform_import_full_backup(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    db_rc: &Rc<RefCell<PhoneticDatabase>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    if let Some(src) = rfd::FileDialog::new()
        .set_title("Import Full Lekhani Backup")
        .add_filter("Lekhani Backup (*.json)", &["json"])
        .pick_file()
    {
        let mut cm = cm_rc.borrow_mut();
        match cm.import_backup(&src) {
            Ok(_) => {
                if let Some(app) = app_opt {
                    apply_settings_to_app(app, &cm.config);
                    app.set_settings_status_text("✓ Backup bundle restored successfully!".into());
                    refresh_stats_ui(app, &cm);
                }
                if let Some(s) = s_opt {
                    apply_settings_to_standalone(s, &cm.config);
                    s.set_settings_status_text("✓ Backup bundle restored successfully!".into());
                    refresh_standalone_stats_ui(s, &cm);
                }
                let p = cm.get_user_learned_path();
                let learner = lekhani_core::AutonomousLearner::load_from_path(&p);
                let stats_str = format!(
                    "{} learned words • {} phrases",
                    learner.learned_words.len(),
                    learner.user_bigrams.len()
                );
                if let Some(app) = app_opt {
                    app.set_learned_stats_text(stats_str.clone().into());
                }
                if let Some(s) = s_opt {
                    s.set_learned_stats_text(stats_str.into());
                }

                // Reload user autocorrect cache & stats
                let ac_path = cm.get_user_autocorrect_path();
                let mut db = db_rc.borrow_mut();
                db.load_user_autocorrect(&ac_path);
                let total_entries =
                    db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
                let ac_status = format!(
                    "Total active entries: {} (User: {})",
                    total_entries,
                    db.get_user_autocorrect().len()
                );
                if let Some(app) = app_opt {
                    app.set_ac_status_text(ac_status.clone().into());
                }
                if let Some(s) = s_opt {
                    s.set_ac_status_text(ac_status.into());
                }
            }
            Err(e) => {
                let err_msg = format!("Import failed: {}", e);
                if let Some(app) = app_opt {
                    app.set_settings_status_text(err_msg.clone().into());
                }
                if let Some(s) = s_opt {
                    s.set_settings_status_text(err_msg.into());
                }
            }
        }
    }
}

pub fn perform_export_learned_data(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    if let Some(dest) = rfd::FileDialog::new()
        .set_title("Export Learned Vocabulary & Phrases")
        .add_filter("JSON (*.json)", &["json"])
        .set_file_name("lekhani_learned_words.json")
        .save_file()
    {
        let cm = cm_rc.borrow();
        let p = cm.get_user_learned_path();
        let res = if p.exists() {
            std::fs::copy(&p, &dest)
                .map(|_| ())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        } else {
            let mut empty_learner = lekhani_core::AutonomousLearner::load_from_path(&p);
            empty_learner.dirty = true;
            empty_learner
                .save_to_path(&dest)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        };
        let msg = match res {
            Ok(_) => "✓ Learned vocabulary & phrases exported!".to_string(),
            Err(e) => format!("Export failed: {}", e),
        };
        if let Some(app) = app_opt {
            app.set_settings_status_text(msg.clone().into());
        }
        if let Some(s) = s_opt {
            s.set_settings_status_text(msg.into());
        }
    }
}

pub fn perform_import_learned_data(
    cm_rc: &Rc<RefCell<ConfigManager>>,
    app_opt: Option<&TopBarWindow>,
    s_opt: Option<&StandaloneDialogWindow>,
) {
    if let Some(src) = rfd::FileDialog::new()
        .set_title("Import Learned Vocabulary & Phrases")
        .add_filter("Binary/JSON (*.bin, *.json)", &["bin", "json"])
        .pick_file()
    {
        let mut loaded = lekhani_core::AutonomousLearner::load_from_path(&src);
        loaded.dirty = true;
        let cm = cm_rc.borrow();
        let p = cm.get_user_learned_path();
        match loaded.save_to_path(&p) {
            Ok(_) => {
                let stats_str = format!(
                    "{} learned words • {} phrases",
                    loaded.learned_words.len(),
                    loaded.user_bigrams.len()
                );
                if let Some(app) = app_opt {
                    app.set_learned_stats_text(stats_str.clone().into());
                    app.set_settings_status_text("✓ Learned data imported successfully!".into());
                }
                if let Some(s) = s_opt {
                    s.set_learned_stats_text(stats_str.into());
                    s.set_settings_status_text("✓ Learned data imported successfully!".into());
                }
            }
            Err(e) => {
                let err_msg = format!("Import failed: {}", e);
                if let Some(app) = app_opt {
                    app.set_settings_status_text(err_msg.clone().into());
                }
                if let Some(s) = s_opt {
                    s.set_settings_status_text(err_msg.into());
                }
            }
        }
    }
}
