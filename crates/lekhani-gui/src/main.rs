#![windows_subsystem = "windows"]
//! Lekhani Desktop GUI Application

slint::include_modules!();

#[cfg(unix)]
mod tray;

#[cfg(windows)]
mod win_autostart;
#[cfg(windows)]
mod win_candidate;
#[cfg(windows)]
mod win_hook;
#[cfg(windows)]
mod win_osd;
#[cfg(windows)]
mod win_tray;

use lekhani_core::{
    bijoy_to_unicode, unicode_to_bijoy, ConjunctCatalog, PhoneticDatabase, PhoneticSuggestion,
    UserStats,
};
use lekhani_settings::{ConfigManager, LayoutManager};
use std::rc::Rc;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Lekhani GUI Desktop Suite...");

    let app = TopBarWindow::new()?;
    let app_weak = app.as_weak();

    let standalone = StandaloneDialogWindow::new()?;
    let standalone_weak = standalone.as_weak();

    let config_mgr = ConfigManager::new();
    let mut layout_mgr = LayoutManager::new();
    let mut db = PhoneticDatabase::new();

    let system_dir = ConfigManager::get_system_layout_dir();
    let user_dir = config_mgr.get_user_layout_dir();
    layout_mgr.discover_layouts(system_dir, user_dir);

    let system_data = ConfigManager::get_system_data_dir();
    let _ = db.load_from_dir(system_data);
    let user_ac_path = config_mgr.get_user_autocorrect_path();
    db.load_user_autocorrect(&user_ac_path);

    // Initialize UI state
    let current_layout = config_mgr.config.general.active_layout.clone();
    app.set_active_layout_name(current_layout.clone().into());
    standalone.set_active_layout_name(current_layout.clone().into());
    #[cfg(windows)]
    app.set_is_bengali_mode(false);
    #[cfg(not(windows))]
    {
        let is_fcitx5_active = std::process::Command::new("fcitx5-remote")
            .arg("-n")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "lekhani")
            .unwrap_or(true);
        app.set_is_bengali_mode(is_fcitx5_active);
    }
    let layout_mgr_rc = std::rc::Rc::new(std::cell::RefCell::new(layout_mgr));
    let layouts = layout_mgr_rc.borrow().get_layout_list();
    let slint_layouts: Vec<slint::SharedString> = layouts.into_iter().map(|s| s.into()).collect();
    let model = std::rc::Rc::new(slint::VecModel::from(slint_layouts));
    app.set_available_layouts(model.clone().into());
    standalone.set_available_layouts(model.into());

    update_viewer_ui(&app, &layout_mgr_rc.borrow(), &current_layout, 0);
    update_standalone_viewer_ui(&standalone, &layout_mgr_rc.borrow(), &current_layout, 0);

    let args: Vec<String> = std::env::args().collect();
    let is_standalone = args.contains(&"--standalone".to_string());
    if args.contains(&"--converter".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(4);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(4);
        }
    } else if args.contains(&"--settings".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(5);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(5);
        }
    } else if args.contains(&"--autocorrect".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(3);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(3);
        }
    } else if args.contains(&"--viewer".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(2);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(2);
        }
    } else if args.contains(&"--layout-menu".to_string()) {
        app.set_active_dialog(1);
    }

    let total_entries = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
    app.set_ac_status_text(
        format!(
            "Total active entries: {} (User: {})",
            total_entries,
            db.get_user_autocorrect().len()
        )
        .into(),
    );

    // Settings state
    #[cfg(windows)]
    {
        app.set_is_windows(true);
        app.set_set_autostart(win_autostart::is_autostart_enabled());
    }
    app.set_set_use_dict(config_mgr.config.phonetic.use_dictionary);
    app.set_set_include_eng(config_mgr.config.phonetic.include_english);
    app.set_set_enter_closes(config_mgr.config.phonetic.enter_key_closes_candidate_window);
    app.set_set_predictive_next(config_mgr.config.phonetic.enable_predictive_next_words);
    app.set_set_auto_vowel(config_mgr.config.fixed.auto_vowel_forming);
    app.set_set_auto_chandra(config_mgr.config.fixed.auto_chandra_position);
    app.set_set_traditional_kar(config_mgr.config.fixed.traditional_kar);
    app.set_set_old_reph(config_mgr.config.fixed.old_reph);
    app.set_set_numberpad(config_mgr.config.fixed.numberpad);
    app.set_set_toggle_key(config_mgr.config.general.toggle_key.clone().into());
    app.set_set_show_osd(config_mgr.config.general.show_osd);
    app.set_set_auto_dari(config_mgr.config.general.auto_dari);

    standalone.set_set_auto_vowel(config_mgr.config.fixed.auto_vowel_forming);
    standalone.set_set_auto_chandra(config_mgr.config.fixed.auto_chandra_position);
    standalone.set_set_traditional_kar(config_mgr.config.fixed.traditional_kar);
    standalone.set_set_old_reph(config_mgr.config.fixed.old_reph);
    standalone.set_set_numberpad(config_mgr.config.fixed.numberpad);
    standalone.set_set_toggle_key(config_mgr.config.general.toggle_key.clone().into());
    standalone.set_set_show_osd(config_mgr.config.general.show_osd);
    standalone.set_set_auto_dari(config_mgr.config.general.auto_dari);

    // Initialize Conjunct Assistant state
    let all_conjuncts = ConjunctCatalog::all();
    let slint_conjuncts: Vec<ConjunctEntry> = all_conjuncts
        .iter()
        .map(|c| ConjunctEntry {
            conjunct: c.conjunct.clone().into(),
            breakdown: c.breakdown.clone().into(),
            phonetic: c.phonetic.clone().into(),
            examples: c.examples.clone().into(),
        })
        .collect();
    let conjunct_model = Rc::new(slint::VecModel::from(slint_conjuncts));
    app.set_conjunct_list(conjunct_model.clone().into());
    standalone.set_conjunct_list(conjunct_model.into());

    // Initialize Typing Telemetry & Analytics state
    refresh_stats_ui(&app, &config_mgr);
    refresh_standalone_stats_ui(&standalone, &config_mgr);

    // Initialize Desktop System Tray (StatusNotifierItem on Linux, Shell_NotifyIconW on Windows)
    #[cfg(unix)]
    let tray_handle = tray::spawn_tray(current_layout.clone(), app_weak.clone());

    #[cfg(windows)]
    let win_tray_handle = win_tray::spawn_windows_tray(current_layout.clone());
    #[cfg(windows)]
    win_hook::spawn_windows_hook(current_layout.clone(), win_tray_handle);

    // Set Native Window Icon on Winit Window
    use i_slint_backend_winit::WinitWindowAccessor;
    let icon_bytes = include_bytes!("../../../data/icons/128.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.into_rgba8();
        let (w, h) = rgba.dimensions();
        let raw = rgba.into_raw();
        let _ = app.window().with_winit_window(move |winit_window| {
            if let Ok(icon) = i_slint_backend_winit::winit::window::Icon::from_rgba(raw, w, h) {
                winit_window.set_window_icon(Some(icon));
            }
            winit_window
                .set_window_level(i_slint_backend_winit::winit::window::WindowLevel::AlwaysOnTop);
        });
    }

    // Callbacks: Window Dragging & Movement (Wayland & X11 Native Compositor Drag)
    let app_weak_drag = app_weak.clone();
    app.on_start_drag_window(move || {
        if let Some(app) = app_weak_drag.upgrade() {
            let _ = app.window().with_winit_window(|winit_window| {
                let _ = winit_window.drag_window();
            });
            // Reset Slint's internal pointer grab state asynchronously so buttons remain responsive
            let app_weak_reset = app_weak_drag.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = app_weak_reset.upgrade() {
                    let _ = app.window().try_dispatch_event(
                        slint::platform::WindowEvent::PointerReleased {
                            position: slint::LogicalPosition::new(0.0, 0.0),
                            button: slint::platform::PointerEventButton::Left,
                        },
                    );
                    let _ = app
                        .window()
                        .try_dispatch_event(slint::platform::WindowEvent::PointerExited);
                }
            });
        }
    });

    let app_weak_end = app_weak.clone();
    app.on_end_drag_window(move || {
        if let Some(app) = app_weak_end.upgrade() {
            let _ =
                app.window()
                    .try_dispatch_event(slint::platform::WindowEvent::PointerReleased {
                        position: slint::LogicalPosition::new(0.0, 0.0),
                        button: slint::platform::PointerEventButton::Left,
                    });
            let _ = app
                .window()
                .try_dispatch_event(slint::platform::WindowEvent::PointerExited);
        }
    });

    // OSD Toast Timer
    let osd_timer = Rc::new(std::cell::RefCell::new(slint::Timer::default()));

    let app_weak_mode_notify = app_weak.clone();
    let osd_timer_mode = osd_timer.clone();
    app.on_notify_mode_change(move |active, layout| {
        if let Some(app) = app_weak_mode_notify.upgrade() {
            app.set_is_bengali_mode(active);
            if active {
                app.set_active_layout_name(layout.clone());
                show_mode_osd(&app, &osd_timer_mode, layout.as_str(), true);
            } else {
                show_mode_osd(&app, &osd_timer_mode, "English", false);
            }
        }
    });

    #[cfg(windows)]
    {
        let app_weak_hook = app_weak.clone();
        win_hook::set_mode_change_callback(std::sync::Arc::new(move |active, layout_name| {
            let app_weak = app_weak_hook.clone();
            let layout: slint::SharedString = layout_name.into();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = app_weak.upgrade() {
                    app.invoke_notify_mode_change(active, layout);
                }
            });
        }));
    }

    // Callbacks: Layout Switching & Mode OSD
    let config_mgr_rc = Rc::new(std::cell::RefCell::new(config_mgr));
    let cm_clone = config_mgr_rc.clone();
    let app_weak_layout = app_weak.clone();
    let standalone_weak_layout = standalone_weak.clone();
    let osd_timer_layout = osd_timer.clone();
    let lm_layout = layout_mgr_rc.clone();
    #[cfg(unix)]
    let tray_handle_layout = tray_handle.clone();
    app.on_select_layout(move |name| {
        let mut cm = cm_clone.borrow_mut();
        cm.config.general.active_layout = name.to_string();
        cm.save();
        #[cfg(windows)]
        {
            win_hook::update_active_layout(&name);
            win_hook::set_bengali_mode(true);
        }
        #[cfg(unix)]
        if let Some(ref th) = tray_handle_layout {
            th.update_layout(&name);
        }
        if let Some(app) = app_weak_layout.upgrade() {
            app.set_active_layout_name(name.clone());
            app.set_viewer_selected_layout(name.clone());
            app.set_show_layout_menu(false);
            show_mode_osd(&app, &osd_timer_layout, &name, true);
            let lm = lm_layout.borrow();
            update_viewer_ui(&app, &lm, &name, app.get_viewer_mode());
        }
        if let Some(s) = standalone_weak_layout.upgrade() {
            s.set_active_layout_name(name.clone());
            let lm = lm_layout.borrow();
            let sel = s.get_viewer_selected_layout();
            if sel.is_empty() || sel == name {
                s.set_viewer_selected_layout(name.clone());
                update_standalone_viewer_ui(&s, &lm, &name, s.get_viewer_mode());
            }
        }
    });

    #[cfg(not(windows))]
    let cm_toggle = config_mgr_rc.clone();
    #[cfg(not(windows))]
    let app_weak_toggle = app_weak.clone();
    #[cfg(not(windows))]
    let osd_timer_toggle = osd_timer.clone();
    #[cfg(unix)]
    let tray_handle_toggle = tray_handle.clone();
    app.on_toggle_layout_mode(move || {
        #[cfg(windows)]
        win_hook::toggle_bengali_mode();

        #[cfg(not(windows))]
        if let Some(app) = app_weak_toggle.upgrade() {
            let is_bengali = app.get_is_bengali_mode();
            let new_mode = !is_bengali;
            app.set_is_bengali_mode(new_mode);
            let cm = cm_toggle.borrow();
            let layout = cm.config.general.active_layout.clone();
            if new_mode {
                #[cfg(unix)]
                if let Some(ref th) = tray_handle_toggle {
                    th.update_layout(&layout);
                }
                show_mode_osd(&app, &osd_timer_toggle, &layout, true);
                let _ = std::process::Command::new("fcitx5-remote")
                    .arg("-s")
                    .arg("lekhani")
                    .spawn();
            } else {
                #[cfg(unix)]
                if let Some(ref th) = tray_handle_toggle {
                    th.update_layout("English");
                }
                show_mode_osd(&app, &osd_timer_toggle, "English", false);
                let _ = std::process::Command::new("fcitx5-remote")
                    .arg("-s")
                    .arg("keyboard-us")
                    .spawn();
            }
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
    let db_rc = Rc::new(std::cell::RefCell::new(db));
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

    // Callbacks: AutoCorrect Add / Update / Delete
    let db_add_clone = db_rc.clone();
    let cm_ac_clone = config_mgr_rc.clone();
    let app_weak_add = app_weak.clone();
    app.on_ac_add_or_update(move |trigger, replacement| {
        if trigger.is_empty() || replacement.is_empty() {
            return;
        }
        let mut db = db_add_clone.borrow_mut();
        db.insert_user_autocorrect(trigger.to_string(), replacement.to_string());
        let cm = cm_ac_clone.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        if let Some(app) = app_weak_add.upgrade() {
            let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
            app.set_ac_status_text(
                format!(
                    "Saved! Total entries: {} (User: {})",
                    total,
                    db.get_user_autocorrect().len()
                )
                .into(),
            );
        }
    });

    let db_del_clone = db_rc.clone();
    let cm_del_clone = config_mgr_rc.clone();
    let app_weak_del = app_weak.clone();
    app.on_ac_delete_entry(move |trigger| {
        let mut db = db_del_clone.borrow_mut();
        db.remove_user_autocorrect(&trigger);
        let cm = cm_del_clone.borrow();
        let user_ac_path = cm.get_user_autocorrect_path();
        let _ = db.save_user_autocorrect(&user_ac_path);
        if let Some(app) = app_weak_del.upgrade() {
            let total = db.get_user_autocorrect().len() + db.get_system_autocorrect().len();
            app.set_ac_status_text(
                format!(
                    "Removed. Total entries: {} (User: {})",
                    total,
                    db.get_user_autocorrect().len()
                )
                .into(),
            );
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

    // Callbacks: Conjunct Assistant
    let app_weak_conj = app_weak.clone();
    app.on_conjunct_search_changed(move |query| {
        let results = if query.is_empty() {
            ConjunctCatalog::all()
        } else {
            ConjunctCatalog::search(query.as_str())
        };
        let slint_results: Vec<ConjunctEntry> = results
            .into_iter()
            .map(|c| ConjunctEntry {
                conjunct: c.conjunct.into(),
                breakdown: c.breakdown.into(),
                phonetic: c.phonetic.into(),
                examples: c.examples.into(),
            })
            .collect();
        if let Some(app) = app_weak_conj.upgrade() {
            app.set_conjunct_list(Rc::new(slint::VecModel::from(slint_results)).into());
        }
    });

    let copy_text = Rc::new(|text: &str| {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::DataExchange::{
                CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
            };
            use windows_sys::Win32::System::Memory::{
                GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
            };
            unsafe {
                if OpenClipboard(std::ptr::null_mut()) != 0 {
                    EmptyClipboard();
                    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
                    let bytes = wide.len() * 2;
                    let h_mem = GlobalAlloc(GMEM_MOVEABLE, bytes);
                    if !h_mem.is_null() {
                        let ptr = GlobalLock(h_mem) as *mut u16;
                        if !ptr.is_null() {
                            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
                            GlobalUnlock(h_mem);
                            SetClipboardData(13 /* CF_UNICODETEXT */, h_mem);
                        }
                    }
                    CloseClipboard();
                }
            }
        }
        #[cfg(unix)]
        {
            let res = std::process::Command::new("wl-copy").arg(text).spawn();
            if res.is_err() {
                use std::io::Write;
                if let Ok(mut child) = std::process::Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                }
            }
        }
    });

    let copy_fn1 = copy_text.clone();
    app.on_copy_conjunct(move |conjunct| {
        copy_fn1(conjunct.as_str());
    });

    let app_weak_conv = app_weak.clone();
    let copy_fn2 = copy_text.clone();
    app.on_copy_to_clipboard(move |text| {
        copy_fn2(text.as_str());
        if let Some(app) = app_weak_conv.upgrade() {
            app.set_conv_copied(true);
        }
    });

    // Detach Dialog Callback on TopBarWindow
    let standalone_weak_detach = standalone_weak.clone();
    let app_weak_detach = app_weak.clone();
    let cm_detach = config_mgr_rc.clone();
    let lm_detach = layout_mgr_rc.clone();
    app.on_detach_dialog(move |dialog_id| {
        if let Some(app) = app_weak_detach.upgrade() {
            app.set_active_dialog(0);
        }
        if let Some(standalone) = standalone_weak_detach.upgrade() {
            standalone.set_dialog_type(dialog_id);
            if dialog_id == 2 {
                let lm = lm_detach.borrow();
                let sel = standalone.get_viewer_selected_layout();
                let layout_name = if sel.is_empty() {
                    cm_detach.borrow().config.general.active_layout.clone()
                } else {
                    sel.to_string()
                };
                update_standalone_viewer_ui(&standalone, &lm, &layout_name, standalone.get_viewer_mode());
            } else if dialog_id == 8 {
                let cm = cm_detach.borrow();
                refresh_standalone_stats_ui(&standalone, &cm);
            }
            let _ = standalone.show();
        }
    });

    // Close Callback on StandaloneDialogWindow
    let standalone_weak_close = standalone_weak.clone();
    standalone.on_close_window(move || {
        if let Some(s) = standalone_weak_close.upgrade() {
            let _ = s.hide();
        }
    });

    // Layout Viewer Callbacks on TopBarWindow
    let app_weak_vm = app_weak.clone();
    let lm_vm = layout_mgr_rc.clone();
    app.on_viewer_mode_changed(move |mode| {
        if let Some(app) = app_weak_vm.upgrade() {
            let mut layout_name = app.get_viewer_selected_layout().to_string();
            if layout_name.is_empty() {
                layout_name = app.get_active_layout_name().to_string();
            }
            let lm = lm_vm.borrow();
            update_viewer_ui(&app, &lm, &layout_name, mode);
        }
    });

    let app_weak_vl = app_weak.clone();
    let lm_vl = layout_mgr_rc.clone();
    app.on_select_viewer_layout(move |layout_name| {
        if let Some(app) = app_weak_vl.upgrade() {
            let lm = lm_vl.borrow();
            update_viewer_ui(&app, &lm, &layout_name, app.get_viewer_mode());
        }
    });

    // Layout Viewer Callbacks on StandaloneDialogWindow
    let s_weak_vm = standalone_weak.clone();
    let lm_svm = layout_mgr_rc.clone();
    standalone.on_viewer_mode_changed(move |mode| {
        if let Some(s) = s_weak_vm.upgrade() {
            let mut layout_name = s.get_viewer_selected_layout().to_string();
            if layout_name.is_empty() {
                layout_name = s.get_active_layout_name().to_string();
            }
            let lm = lm_svm.borrow();
            update_standalone_viewer_ui(&s, &lm, &layout_name, mode);
        }
    });

    let s_weak_vl = standalone_weak.clone();
    let lm_svl = layout_mgr_rc.clone();
    standalone.on_select_viewer_layout(move |layout_name| {
        if let Some(s) = s_weak_vl.upgrade() {
            let lm = lm_svl.borrow();
            update_standalone_viewer_ui(&s, &lm, &layout_name, s.get_viewer_mode());
        }
    });

    // Standalone Converter Callbacks
    let s_weak_c1 = standalone_weak.clone();
    standalone.on_convert_bijoy_to_unicode(move |text| {
        let converted = bijoy_to_unicode(&text);
        if let Some(s) = s_weak_c1.upgrade() {
            s.set_conv_output_text(converted.into());
        }
    });

    let s_weak_c2 = standalone_weak.clone();
    standalone.on_convert_unicode_to_bijoy(move |text| {
        let converted = unicode_to_bijoy(&text);
        if let Some(s) = s_weak_c2.upgrade() {
            s.set_conv_output_text(converted.into());
        }
    });

    let s_weak_copy = standalone_weak.clone();
    let copy_fn_sa = copy_text.clone();
    standalone.on_copy_to_clipboard(move |text| {
        copy_fn_sa(text.as_str());
        if let Some(s) = s_weak_copy.upgrade() {
            s.set_conv_copied(true);
        }
    });

    let copy_fn_sa_conj = copy_text.clone();
    standalone.on_copy_conjunct(move |conjunct| {
        copy_fn_sa_conj(conjunct.as_str());
    });

    let s_weak_conj = standalone_weak.clone();
    standalone.on_conjunct_search_changed(move |query| {
        let results = if query.is_empty() {
            ConjunctCatalog::all()
        } else {
            ConjunctCatalog::search(query.as_str())
        };
        let slint_results: Vec<ConjunctEntry> = results
            .into_iter()
            .map(|c| ConjunctEntry {
                conjunct: c.conjunct.into(),
                breakdown: c.breakdown.into(),
                phonetic: c.phonetic.into(),
                examples: c.examples.into(),
            })
            .collect();
        if let Some(s) = s_weak_conj.upgrade() {
            s.set_conjunct_list(Rc::new(slint::VecModel::from(slint_results)).into());
        }
    });

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
    let s_weak_add = standalone_weak.clone();
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
        if let Some(s) = s_weak_add.upgrade() {
            s.set_ac_status_text(status.clone().into());
        }
        if let Some(app) = app_weak_add2.upgrade() {
            app.set_ac_status_text(status.into());
        }
    });

    let db_del_clone2 = db_rc.clone();
    let cm_del_clone2 = config_mgr_rc.clone();
    let s_weak_del = standalone_weak.clone();
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
        if let Some(s) = s_weak_del.upgrade() {
            s.set_ac_status_text(status.clone().into());
        }
        if let Some(app) = app_weak_del2.upgrade() {
            app.set_ac_status_text(status.into());
        }
    });

    let cm_save2 = config_mgr_rc.clone();
    let s_weak_set = standalone_weak.clone();
    standalone.on_save_settings(move || {
        if let Some(s) = s_weak_set.upgrade() {
            let mut cm = cm_save2.borrow_mut();
            cm.config.fixed.auto_vowel_forming = s.get_set_auto_vowel();
            cm.config.fixed.auto_chandra_position = s.get_set_auto_chandra();
            cm.config.fixed.traditional_kar = s.get_set_traditional_kar();
            cm.config.fixed.old_reph = s.get_set_old_reph();
            cm.config.fixed.numberpad = s.get_set_numberpad();
            cm.config.general.toggle_key = s.get_set_toggle_key().to_string();
            cm.config.general.show_osd = s.get_set_show_osd();
            cm.config.general.auto_dari = s.get_set_auto_dari();
            cm.save();
            let _ = s.hide();
        }
    });

    let s_weak_stats = standalone_weak.clone();
    let cm_stats2 = config_mgr_rc.clone();
    standalone.on_refresh_stats(move || {
        if let Some(s) = s_weak_stats.upgrade() {
            let cm = cm_stats2.borrow();
            refresh_standalone_stats_ui(&s, &cm);
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
            cm.config.phonetic.enable_predictive_next_words = app.get_set_predictive_next();
            cm.config.fixed.auto_vowel_forming = app.get_set_auto_vowel();
            cm.config.fixed.auto_chandra_position = app.get_set_auto_chandra();
            cm.config.fixed.traditional_kar = app.get_set_traditional_kar();
            cm.config.fixed.old_reph = app.get_set_old_reph();
            cm.config.fixed.numberpad = app.get_set_numberpad();
            cm.config.general.toggle_key = app.get_set_toggle_key().to_string();
            cm.config.general.show_osd = app.get_set_show_osd();
            cm.config.general.auto_dari = app.get_set_auto_dari();
            cm.save();
            #[cfg(windows)]
            let _ = win_autostart::set_autostart(app.get_set_autostart());
            app.set_show_settings_dialog(false);
        }
    });

    // Callbacks: Typing Stats Live Refresh
    let app_weak_stats = app_weak.clone();
    let cm_stats = config_mgr_rc.clone();
    app.on_refresh_stats(move || {
        if let Some(app) = app_weak_stats.upgrade() {
            let cm = cm_stats.borrow();
            refresh_stats_ui(&app, &cm);
        }
    });

    let app_weak_stats_timer = app_weak.clone();
    let cm_stats_timer = config_mgr_rc.clone();
    #[cfg(unix)]
    let tray_handle_timer = tray_handle.clone();
    let stats_poll_timer = Rc::new(std::cell::RefCell::new(slint::Timer::default()));
    stats_poll_timer.borrow_mut().start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(1000),
        move || {
            if let Some(app) = app_weak_stats_timer.upgrade() {
                let cm = cm_stats_timer.borrow();
                if app.get_active_dialog() == 8 {
                    refresh_stats_ui(&app, &cm);
                }

                #[cfg(unix)]
                {
                    if let Ok(out) = std::process::Command::new("fcitx5-remote")
                        .arg("-n")
                        .output()
                    {
                        let im_name = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        let is_bengali = im_name == "lekhani";
                        if app.get_is_bengali_mode() != is_bengali {
                            app.set_is_bengali_mode(is_bengali);
                            let layout = cm.config.general.active_layout.clone();
                            if let Some(ref th) = tray_handle_timer {
                                th.update_layout(if is_bengali { &layout } else { "English" });
                            }
                        }
                    }
                }
            }
        },
    );

    // Callbacks: Quit
    app.on_trigger_quit(move || {
        let _ = slint::quit_event_loop();
    });

    app.run()?;
    Ok(())
}

fn refresh_stats_ui(app: &TopBarWindow, config_mgr: &ConfigManager) {
    let user_stats_path = config_mgr.get_user_stats_path();
    let stats = UserStats::load_from_path(&user_stats_path);
    app.set_stats_total_words(stats.total_words_typed as i32);
    app.set_stats_keystrokes_saved(stats.keystrokes_saved as i32);
    app.set_stats_efficiency_ratio(format!("{:.1}%", stats.savings_percentage()).into());

    let top_chars = stats.get_top_characters(10);
    let slint_top_chars: Vec<HeatmapEntry> = top_chars
        .into_iter()
        .map(|(ch, count, pct)| HeatmapEntry {
            character: ch.to_string().into(),
            count: count as i32,
            percentage: format!("{:.1}%", pct).into(),
        })
        .collect();
    app.set_stats_top_chars(Rc::new(slint::VecModel::from(slint_top_chars)).into());
}

fn refresh_standalone_stats_ui(standalone: &StandaloneDialogWindow, config_mgr: &ConfigManager) {
    let user_stats_path = config_mgr.get_user_stats_path();
    let stats = UserStats::load_from_path(&user_stats_path);
    standalone.set_stats_total_words(stats.total_words_typed as i32);
    standalone.set_stats_keystrokes_saved(stats.keystrokes_saved as i32);

    let top_chars = stats.get_top_characters(10);
    let slint_top_chars: Vec<HeatmapEntry> = top_chars
        .into_iter()
        .map(|(ch, count, pct)| HeatmapEntry {
            character: ch.to_string().into(),
            count: count as i32,
            percentage: format!("{:.1}%", pct).into(),
        })
        .collect();
    standalone.set_stats_top_characters(Rc::new(slint::VecModel::from(slint_top_chars)).into());
}

fn show_mode_osd(
    app: &TopBarWindow,
    timer: &Rc<std::cell::RefCell<slint::Timer>>,
    layout_name: &str,
    is_bengali: bool,
) {
    app.set_osd_title(layout_name.into());
    if is_bengali {
        app.set_osd_subtitle("বাংলা মোড সক্রিয় (Bengali Active)".into());
        app.set_osd_icon("🇧🇩".into());
        app.set_osd_badge_color(slint::Color::from_rgb_u8(166, 227, 161)); // #a6e3a1 Green
        app.set_osd_badge_bg(slint::Color::from_argb_u8(40, 166, 227, 161));
    } else {
        app.set_osd_subtitle("English Mode Active (F12 to switch)".into());
        app.set_osd_icon("🌐".into());
        app.set_osd_badge_color(slint::Color::from_rgb_u8(137, 180, 250)); // #89b4fa Blue
        app.set_osd_badge_bg(slint::Color::from_argb_u8(40, 137, 180, 250));
    }
    app.set_show_osd(true);

    let app_weak = app.as_weak();
    timer.borrow_mut().start(
        slint::TimerMode::SingleShot,
        std::time::Duration::from_millis(1800),
        move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_show_osd(false);
            }
        },
    );
}

fn map_spec(
    layout: Option<&serde_json::Map<String, serde_json::Value>>,
    mode: i32,
    spec: &[(&str, &str, &str, &str)],
) -> Vec<String> {
    spec.iter()
        .map(|(norm_k, shift_k, alt_k, def)| {
            if let Some(map) = layout {
                let val = match mode {
                    0 => map.get(*norm_k).and_then(|v| v.as_str()),
                    1 => map.get(*shift_k).and_then(|v| v.as_str()),
                    _ => map
                        .get(*alt_k)
                        .and_then(|v| v.as_str())
                        .or_else(|| map.get(*norm_k).and_then(|v| v.as_str())),
                };
                if let Some(v) = val {
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
            def.to_string()
        })
        .collect()
}

fn get_layout_rows(
    layout_mgr: &LayoutManager,
    layout_name: &str,
    mode: i32,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let clean = layout_name.to_lowercase();
    let is_phonetic = clean.contains("phonetic")
        || layout_mgr
            .get_layout(layout_name)
            .map(|i| i.layout_type == "phonetic")
            .unwrap_or(false);

    if is_phonetic {
        let r1 = match mode {
            0 => vec!["`", "১", "২", "৩", "৪", "৫", "৬", "৭", "৮", "৯", "০", "-", "="],
            1 => vec!["~", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "_", "+"],
            _ => vec!["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "–", "≠"],
        }
        .into_iter()
        .map(String::from)
        .collect();

        let r2 = match mode {
            0 => vec!["ক", "ও", "এ", "র", "ট", "য়", "উ", "ই", "ও", "প", "[", "]", "\\"],
            1 => vec!["ক", "ঢ়", "ঈ", "ড়", "ঠ", "য়", "ঊ", "ঈ", "ঔ", "ফ", "{", "}", "|"],
            _ => vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]", "\\"],
        }
        .into_iter()
        .map(String::from)
        .collect();

        let r3 = match mode {
            0 => vec!["আ", "স", "ড", "ফ", "গ", "হ", "জ", "ক", "ল", ";", "'"],
            1 => vec!["অ", "শ", "ঢ", "ফ", "ঘ", "ঃ", "ঝ", "খ", "ল", ":", "\""],
            _ => vec!["a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'"],
        }
        .into_iter()
        .map(String::from)
        .collect();

        let r4 = match mode {
            0 => vec!["য", "ক্স", "চ", "ভ", "ব", "ন", "ম", ",", ".", "/"],
            1 => vec!["য", "ক্স", "ছ", "ভ", "ভ", "ণ", "ং", "<", ">", "?"],
            _ => vec!["z", "x", "c", "v", "b", "n", "m", "<", ">", "?"],
        }
        .into_iter()
        .map(String::from)
        .collect();

        return (r1, r2, r3, r4);
    }

    let json_val = layout_mgr.load_layout_json(layout_name);
    let map = json_val
        .as_ref()
        .and_then(|v| v.get("layout"))
        .and_then(|v| v.as_object());

    let r1_spec = [
        ("Key_Grave_Normal", "Key_Tilde_Normal", "Key_Grave_AltGr", "`"),
        ("Key_1_Normal", "Key_Exclaim_Normal", "Key_1_AltGr", "১"),
        ("Key_2_Normal", "Key_At_Normal", "Key_2_AltGr", "২"),
        ("Key_3_Normal", "Key_Hash_Normal", "Key_3_AltGr", "৩"),
        ("Key_4_Normal", "Key_Dollar_Normal", "Key_4_AltGr", "৪"),
        ("Key_5_Normal", "Key_Percent_Normal", "Key_5_AltGr", "৫"),
        ("Key_6_Normal", "Key_Circum_Normal", "Key_6_AltGr", "৬"),
        ("Key_7_Normal", "Key_Ampersand_Normal", "Key_7_AltGr", "৭"),
        ("Key_8_Normal", "Key_Asterisk_Normal", "Key_8_AltGr", "৮"),
        ("Key_9_Normal", "Key_ParenLeft_Normal", "Key_9_AltGr", "৯"),
        ("Key_0_Normal", "Key_ParenRight_Normal", "Key_0_AltGr", "০"),
        ("Key_Minus_Normal", "Key_UnderScore_Normal", "Key_Minus_AltGr", "-"),
        ("Key_Equals_Normal", "Key_Plus_Normal", "Key_Equals_AltGr", "="),
    ];

    let r2_spec = [
        ("Key_q_Normal", "Key_Q_Normal", "Key_q_AltGr", "q"),
        ("Key_w_Normal", "Key_W_Normal", "Key_w_AltGr", "w"),
        ("Key_e_Normal", "Key_E_Normal", "Key_e_AltGr", "e"),
        ("Key_r_Normal", "Key_R_Normal", "Key_r_AltGr", "r"),
        ("Key_t_Normal", "Key_T_Normal", "Key_t_AltGr", "t"),
        ("Key_y_Normal", "Key_Y_Normal", "Key_y_AltGr", "y"),
        ("Key_u_Normal", "Key_U_Normal", "Key_u_AltGr", "u"),
        ("Key_i_Normal", "Key_I_Normal", "Key_i_AltGr", "i"),
        ("Key_o_Normal", "Key_O_Normal", "Key_o_AltGr", "o"),
        ("Key_p_Normal", "Key_P_Normal", "Key_p_AltGr", "p"),
        ("Key_BracketLeft_Normal", "Key_BraceLeft_Normal", "Key_BracketLeft_AltGr", "["),
        ("Key_BracketRight_Normal", "Key_BraceRight_Normal", "Key_BracketRight_AltGr", "]"),
        ("Key_BackSlash_Normal", "Key_Bar_Normal", "Key_BackSlash_AltGr", "\\"),
    ];

    let r3_spec = [
        ("Key_a_Normal", "Key_A_Normal", "Key_a_AltGr", "a"),
        ("Key_s_Normal", "Key_S_Normal", "Key_s_AltGr", "s"),
        ("Key_d_Normal", "Key_D_Normal", "Key_d_AltGr", "d"),
        ("Key_f_Normal", "Key_F_Normal", "Key_f_AltGr", "f"),
        ("Key_g_Normal", "Key_G_Normal", "Key_g_AltGr", "g"),
        ("Key_h_Normal", "Key_H_Normal", "Key_h_AltGr", "h"),
        ("Key_j_Normal", "Key_J_Normal", "Key_j_AltGr", "j"),
        ("Key_k_Normal", "Key_K_Normal", "Key_k_AltGr", "k"),
        ("Key_l_Normal", "Key_L_Normal", "Key_l_AltGr", "l"),
        ("Key_Semicolon_Normal", "Key_Colon_Normal", "Key_Semicolon_AltGr", ";"),
        ("Key_Apostrophe_Normal", "Key_Quote_Normal", "Key_Apostrophe_AltGr", "'"),
    ];

    let r4_spec = [
        ("Key_z_Normal", "Key_Z_Normal", "Key_z_AltGr", "z"),
        ("Key_x_Normal", "Key_X_Normal", "Key_x_AltGr", "x"),
        ("Key_c_Normal", "Key_C_Normal", "Key_c_AltGr", "c"),
        ("Key_v_Normal", "Key_V_Normal", "Key_v_AltGr", "v"),
        ("Key_b_Normal", "Key_B_Normal", "Key_b_AltGr", "b"),
        ("Key_n_Normal", "Key_N_Normal", "Key_n_AltGr", "n"),
        ("Key_m_Normal", "Key_M_Normal", "Key_m_AltGr", "m"),
        ("Key_Comma_Normal", "Key_Less_Normal", "Key_Comma_AltGr", ","),
        ("Key_Period_Normal", "Key_Greater_Normal", "Key_Period_AltGr", "."),
        ("Key_Slash_Normal", "Key_Question_Normal", "Key_Slash_AltGr", "/"),
    ];

    (
        map_spec(map, mode, &r1_spec),
        map_spec(map, mode, &r2_spec),
        map_spec(map, mode, &r3_spec),
        map_spec(map, mode, &r4_spec),
    )
}

fn update_viewer_ui(app: &TopBarWindow, layout_mgr: &LayoutManager, layout_name: &str, mode: i32) {
    let (r1, r2, r3, r4) = get_layout_rows(layout_mgr, layout_name, mode);
    app.set_viewer_row_1(Rc::new(slint::VecModel::from(r1.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    app.set_viewer_row_2(Rc::new(slint::VecModel::from(r2.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    app.set_viewer_row_3(Rc::new(slint::VecModel::from(r3.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    app.set_viewer_row_4(Rc::new(slint::VecModel::from(r4.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    app.set_viewer_selected_layout(layout_name.into());
    app.set_viewer_mode(mode);
}

fn update_standalone_viewer_ui(standalone: &StandaloneDialogWindow, layout_mgr: &LayoutManager, layout_name: &str, mode: i32) {
    let (r1, r2, r3, r4) = get_layout_rows(layout_mgr, layout_name, mode);
    standalone.set_viewer_row_1(Rc::new(slint::VecModel::from(r1.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    standalone.set_viewer_row_2(Rc::new(slint::VecModel::from(r2.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    standalone.set_viewer_row_3(Rc::new(slint::VecModel::from(r3.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    standalone.set_viewer_row_4(Rc::new(slint::VecModel::from(r4.into_iter().map(slint::SharedString::from).collect::<Vec<_>>())).into());
    standalone.set_viewer_selected_layout(layout_name.into());
    standalone.set_viewer_mode(mode);
}
