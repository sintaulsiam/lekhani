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

mod autocorrect;
mod backup;
mod settings_sync;
mod stats;
mod viewer;

use backup::*;
use settings_sync::*;
use stats::*;
use viewer::*;

use lekhani_core::{bijoy_to_unicode, unicode_to_bijoy, ConjunctCatalog, PhoneticDatabase};
use lekhani_settings::{ConfigManager, LayoutManager};
use std::rc::Rc;
use tracing::info;

#[cfg(unix)]
static FCITX5_AVAILABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

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
        let is_fcitx5_active = match std::process::Command::new("fcitx5-remote").arg("-n").output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "lekhani",
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    FCITX5_AVAILABLE.store(false, std::sync::atomic::Ordering::Relaxed);
                }
                true
            }
        };
        app.set_is_bengali_mode(is_fcitx5_active);

        // Detect if desktop environment lacks Server-Side Decorations (e.g. GNOME on Wayland)
        let is_gnome_wayland = std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_lowercase().contains("gnome"))
            .unwrap_or(false)
            && std::env::var("WAYLAND_DISPLAY").is_ok();
        standalone.set_needs_client_decorations(is_gnome_wayland);
    }
    let layout_mgr_rc = std::rc::Rc::new(std::cell::RefCell::new(layout_mgr));
    let layouts = layout_mgr_rc.borrow().get_layout_list();
    let slint_layouts: Vec<slint::SharedString> = layouts.into_iter().map(|s| s.into()).collect();
    let model = std::rc::Rc::new(slint::VecModel::from(slint_layouts));
    app.set_available_layouts(model.clone().into());
    standalone.set_available_layouts(model.into());

    update_viewer_ui(&app, &layout_mgr_rc.borrow(), &current_layout, 0);
    update_standalone_viewer_ui(&standalone, &layout_mgr_rc.borrow(), &current_layout, 0);

    // Settings state
    #[cfg(windows)]
    {
        app.set_is_windows(true);
        standalone.set_is_windows(true);
        app.set_set_autostart(win_autostart::is_autostart_enabled());
        standalone.set_set_autostart(win_autostart::is_autostart_enabled());
    }
    apply_settings_to_app(&app, &config_mgr.config);
    apply_settings_to_standalone(&standalone, &config_mgr.config);

    // Initialize Autonomous Learning Profile state
    let learned_path = config_mgr.get_user_learned_path();
    let learner_init = lekhani_core::AutonomousLearner::load_from_path(&learned_path);
    let learned_stats_init = format!(
        "{} learned words • {} phrases",
        learner_init.learned_words.len(),
        learner_init.user_bigrams.len()
    );
    app.set_learned_stats_text(learned_stats_init.clone().into());
    standalone.set_learned_stats_text(learned_stats_init.into());

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
        if let Some(pos) = args.iter().position(|a| a == "--tab") {
            if let Some(val) = args.get(pos + 1).and_then(|v| v.parse::<i32>().ok()) {
                if is_standalone {
                    standalone.set_settings_tab(val);
                } else {
                    app.set_settings_tab(val);
                }
            }
        }
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
    } else if args.contains(&"--conjunct".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(7);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(7);
        }
    } else if args.contains(&"--stats".to_string()) {
        if is_standalone {
            standalone.set_dialog_type(8);
            let _ = standalone.show();
        } else {
            app.set_active_dialog(8);
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
    let config_mgr_rc = Rc::new(std::cell::RefCell::new(config_mgr));

    // Initialize Desktop System Tray (StatusNotifierItem on Linux, Shell_NotifyIconW on Windows)
    #[cfg(unix)]
    let tray_handle = tray::spawn_tray(current_layout.clone(), app_weak.clone());

    #[cfg(windows)]
    let win_tray_handle = win_tray::spawn_windows_tray(current_layout.clone(), app_weak.clone());
    #[cfg(windows)]
    win_hook::spawn_windows_hook(current_layout.clone(), win_tray_handle.clone());

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

    // Restore Saved Window Position
    {
        let (saved_x, saved_y) = {
            let cm = config_mgr_rc.borrow();
            (cm.config.ui.topbar_x, cm.config.ui.topbar_y)
        };
        if saved_x > 0 || saved_y > 0 {
            let _ = app.window().with_winit_window(move |winit_window| {
                winit_window.set_outer_position(
                    i_slint_backend_winit::winit::dpi::PhysicalPosition::new(saved_x, saved_y),
                );
            });
        }
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
    let cm_drag_end = config_mgr_rc.clone();
    app.on_end_drag_window(move || {
        if let Some(app) = app_weak_end.upgrade() {
            let _ = app.window().with_winit_window(|winit_window| {
                if let Ok(pos) = winit_window.outer_position() {
                    let mut cm = cm_drag_end.borrow_mut();
                    if cm.config.ui.topbar_x != pos.x || cm.config.ui.topbar_y != pos.y {
                        cm.config.ui.topbar_x = pos.x;
                        cm.config.ui.topbar_y = pos.y;
                        cm.save();
                    }
                }
            });
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

    // Callbacks: AutoCorrect
    let db_rc = Rc::new(std::cell::RefCell::new(db));
    autocorrect::register_autocorrect_callbacks(&app, &standalone, &db_rc, &config_mgr_rc);

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
            } else if dialog_id == 5 {
                let cm = cm_detach.borrow();
                apply_settings_to_standalone(&standalone, &cm.config);
                let learned_path = cm.get_user_learned_path();
                let learner = lekhani_core::AutonomousLearner::load_from_path(&learned_path);
                standalone.set_learned_stats_text(
                    format!(
                        "{} learned words • {} phrases",
                        learner.learned_words.len(),
                        learner.user_bigrams.len()
                    )
                    .into(),
                );
            } else if dialog_id == 8 {
                let cm = cm_detach.borrow();
                refresh_standalone_stats_ui(&standalone, &cm);
            }
            let _ = standalone.show();
        }
    });

    // Close Callback on StandaloneDialogWindow
    let standalone_weak_close = standalone_weak.clone();
    let is_standalone_close = is_standalone;
    standalone.on_close_window(move || {
        if is_standalone_close {
            let _ = slint::quit_event_loop();
        } else if let Some(s) = standalone_weak_close.upgrade() {
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



    // Settings: Standalone Auto-Save
    let cm_auto_s = config_mgr_rc.clone();
    let s_weak_auto = standalone_weak.clone();
    let app_weak_sync = app_weak.clone();
    standalone.on_auto_save_settings(move || {
        if let Some(s) = s_weak_auto.upgrade() {
            let mut cm = cm_auto_s.borrow_mut();
            read_settings_from_standalone(&s, &mut cm.config);
            cm.save();
            #[cfg(windows)]
            let _ = win_autostart::set_autostart(s.get_set_autostart());
            s.set_settings_status_text("✓ All changes saved automatically".into());
            if let Some(app) = app_weak_sync.upgrade() {
                apply_settings_to_app(&app, &cm.config);
                app.set_settings_status_text("✓ All changes saved automatically".into());
            }
        }
    });

    // --- Standalone Data & Backup Management Callbacks ---
    let cm_full_exp_s = config_mgr_rc.clone();
    let s_weak_full_exp = standalone_weak.clone();
    let app_weak_full_exp = app_weak.clone();
    standalone.on_export_full_backup(move || {
        perform_export_full_backup(
            &cm_full_exp_s,
            app_weak_full_exp.upgrade().as_ref(),
            s_weak_full_exp.upgrade().as_ref(),
        );
    });

    let cm_full_imp_s = config_mgr_rc.clone();
    let db_full_imp_s = db_rc.clone();
    let s_weak_full_imp = standalone_weak.clone();
    let app_weak_full_imp = app_weak.clone();
    standalone.on_import_full_backup(move || {
        perform_import_full_backup(
            &cm_full_imp_s,
            &db_full_imp_s,
            app_weak_full_imp.upgrade().as_ref(),
            s_weak_full_imp.upgrade().as_ref(),
        );
    });

    let cm_lrn_exp_s = config_mgr_rc.clone();
    let s_weak_lrn_exp = standalone_weak.clone();
    let app_weak_lrn_exp = app_weak.clone();
    standalone.on_export_learned_data(move || {
        perform_export_learned_data(
            &cm_lrn_exp_s,
            app_weak_lrn_exp.upgrade().as_ref(),
            s_weak_lrn_exp.upgrade().as_ref(),
        );
    });

    let cm_lrn_imp_s = config_mgr_rc.clone();
    let s_weak_lrn_imp = standalone_weak.clone();
    let app_weak_lrn_imp = app_weak.clone();
    standalone.on_import_learned_data(move || {
        perform_import_learned_data(
            &cm_lrn_imp_s,
            app_weak_lrn_imp.upgrade().as_ref(),
            s_weak_lrn_imp.upgrade().as_ref(),
        );
    });

    let s_weak_req_clr = standalone_weak.clone();
    standalone.on_request_clear_learned_data(move || {
        if let Some(s) = s_weak_req_clr.upgrade() {
            s.set_confirm_dialog_type(1);
            s.set_confirm_dialog_title("Reset Learned Vocabulary & Phrases?".into());
            s.set_confirm_dialog_message(
                "This will permanently remove your personalized words, bigram frequencies, and candidate choices. Core dictionary and system idioms will remain intact. This action cannot be undone.".into(),
            );
            s.set_confirm_dialog_btn_text("Yes, Reset Data".into());
            s.set_confirm_is_danger(true);
        }
    });

    let s_weak_req_rst = standalone_weak.clone();
    standalone.on_request_restore_defaults(move || {
        if let Some(s) = s_weak_req_rst.upgrade() {
            s.set_confirm_dialog_type(2);
            s.set_confirm_dialog_title("Restore Default Settings?".into());
            s.set_confirm_dialog_message(
                "This will restore all phonetic and fixed layout options to their factory defaults.".into(),
            );
            s.set_confirm_dialog_btn_text("Restore Defaults".into());
            s.set_confirm_is_danger(false);
        }
    });

    let s_weak_cancel_conf = standalone_weak.clone();
    standalone.on_cancel_confirm_action(move || {
        if let Some(s) = s_weak_cancel_conf.upgrade() {
            s.set_confirm_dialog_type(0);
        }
    });

    let cm_exec_s = config_mgr_rc.clone();
    let s_weak_exec_conf = standalone_weak.clone();
    let app_weak_exec_conf = app_weak.clone();
    standalone.on_execute_confirmed_action(move || {
        if let Some(s) = s_weak_exec_conf.upgrade() {
            let action_type = s.get_confirm_dialog_type();
            s.set_confirm_dialog_type(0);
            if action_type == 1 {
                perform_clear_learned_data(
                    &cm_exec_s,
                    app_weak_exec_conf.upgrade().as_ref(),
                    Some(&s),
                );
            } else if action_type == 2 {
                perform_restore_defaults(
                    &cm_exec_s,
                    app_weak_exec_conf.upgrade().as_ref(),
                    Some(&s),
                );
            }
        }
    });

    let s_weak_reset_legacy = standalone_weak.clone();
    standalone.on_reset_default_settings(move || {
        if let Some(s) = s_weak_reset_legacy.upgrade() {
            s.invoke_request_restore_defaults();
        }
    });

    let s_weak_clear_legacy = standalone_weak.clone();
    standalone.on_clear_learned_data(move || {
        if let Some(s) = s_weak_clear_legacy.upgrade() {
            s.invoke_request_clear_learned_data();
        }
    });

    let s_weak_save_btn = standalone_weak.clone();
    standalone.on_save_settings(move || {
        if let Some(s) = s_weak_save_btn.upgrade() {
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

    // Settings: Popover Auto-Save
    let cm_auto_app = config_mgr_rc.clone();
    let app_weak_auto = app_weak.clone();
    let s_weak_sync = standalone_weak.clone();
    app.on_auto_save_settings(move || {
        if let Some(app) = app_weak_auto.upgrade() {
            let mut cm = cm_auto_app.borrow_mut();
            read_settings_from_app(&app, &mut cm.config);
            cm.save();
            #[cfg(windows)]
            let _ = win_autostart::set_autostart(app.get_set_autostart());
            app.set_settings_status_text("✓ All changes saved automatically".into());
            if let Some(s) = s_weak_sync.upgrade() {
                apply_settings_to_standalone(&s, &cm.config);
                s.set_settings_status_text("✓ All changes saved automatically".into());
            }
        }
    });

    // --- Popover Data & Backup Management Callbacks ---
    let cm_full_exp_app = config_mgr_rc.clone();
    let app_weak_full_exp = app_weak.clone();
    let s_weak_full_exp2 = standalone_weak.clone();
    app.on_export_full_backup(move || {
        perform_export_full_backup(
            &cm_full_exp_app,
            app_weak_full_exp.upgrade().as_ref(),
            s_weak_full_exp2.upgrade().as_ref(),
        );
    });

    let cm_full_imp_app = config_mgr_rc.clone();
    let db_full_imp_app = db_rc.clone();
    let app_weak_full_imp = app_weak.clone();
    let s_weak_full_imp2 = standalone_weak.clone();
    app.on_import_full_backup(move || {
        perform_import_full_backup(
            &cm_full_imp_app,
            &db_full_imp_app,
            app_weak_full_imp.upgrade().as_ref(),
            s_weak_full_imp2.upgrade().as_ref(),
        );
    });

    let cm_lrn_exp_app = config_mgr_rc.clone();
    let app_weak_lrn_exp = app_weak.clone();
    let s_weak_lrn_exp2 = standalone_weak.clone();
    app.on_export_learned_data(move || {
        perform_export_learned_data(
            &cm_lrn_exp_app,
            app_weak_lrn_exp.upgrade().as_ref(),
            s_weak_lrn_exp2.upgrade().as_ref(),
        );
    });

    let cm_lrn_imp_app = config_mgr_rc.clone();
    let app_weak_lrn_imp = app_weak.clone();
    let s_weak_lrn_imp2 = standalone_weak.clone();
    app.on_import_learned_data(move || {
        perform_import_learned_data(
            &cm_lrn_imp_app,
            app_weak_lrn_imp.upgrade().as_ref(),
            s_weak_lrn_imp2.upgrade().as_ref(),
        );
    });

    let app_weak_req_clr = app_weak.clone();
    app.on_request_clear_learned_data(move || {
        if let Some(app) = app_weak_req_clr.upgrade() {
            app.set_confirm_dialog_type(1);
            app.set_confirm_dialog_title("Reset Learned Vocabulary & Phrases?".into());
            app.set_confirm_dialog_message(
                "This will permanently remove your personalized words, bigram frequencies, and candidate choices. Core dictionary and system idioms will remain intact. This action cannot be undone.".into(),
            );
            app.set_confirm_dialog_btn_text("Yes, Reset Data".into());
            app.set_confirm_is_danger(true);
        }
    });

    let app_weak_req_rst = app_weak.clone();
    app.on_request_restore_defaults(move || {
        if let Some(app) = app_weak_req_rst.upgrade() {
            app.set_confirm_dialog_type(2);
            app.set_confirm_dialog_title("Restore Default Settings?".into());
            app.set_confirm_dialog_message(
                "This will restore all phonetic and fixed layout options to their factory defaults.".into(),
            );
            app.set_confirm_dialog_btn_text("Restore Defaults".into());
            app.set_confirm_is_danger(false);
        }
    });

    let app_weak_cancel_conf = app_weak.clone();
    app.on_cancel_confirm_action(move || {
        if let Some(app) = app_weak_cancel_conf.upgrade() {
            app.set_confirm_dialog_type(0);
        }
    });

    let cm_exec_app = config_mgr_rc.clone();
    let app_weak_exec_conf = app_weak.clone();
    let s_weak_exec_conf2 = standalone_weak.clone();
    app.on_execute_confirmed_action(move || {
        if let Some(app) = app_weak_exec_conf.upgrade() {
            let action_type = app.get_confirm_dialog_type();
            app.set_confirm_dialog_type(0);
            if action_type == 1 {
                perform_clear_learned_data(
                    &cm_exec_app,
                    Some(&app),
                    s_weak_exec_conf2.upgrade().as_ref(),
                );
            } else if action_type == 2 {
                perform_restore_defaults(
                    &cm_exec_app,
                    Some(&app),
                    s_weak_exec_conf2.upgrade().as_ref(),
                );
            }
        }
    });

    let app_weak_reset_legacy = app_weak.clone();
    app.on_reset_default_settings(move || {
        if let Some(app) = app_weak_reset_legacy.upgrade() {
            app.invoke_request_restore_defaults();
        }
    });

    let app_weak_clear_legacy = app_weak.clone();
    app.on_clear_learned_data(move || {
        if let Some(app) = app_weak_clear_legacy.upgrade() {
            app.invoke_request_clear_learned_data();
        }
    });

    // Settings: Popover Done / Save
    let app_weak_set = app_weak.clone();
    app.on_save_settings(move || {
        if let Some(app) = app_weak_set.upgrade() {
            app.set_active_dialog(0);
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
                    if FCITX5_AVAILABLE.load(std::sync::atomic::Ordering::Relaxed) {
                        match std::process::Command::new("fcitx5-remote").arg("-n").output() {
                            Ok(out) => {
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
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                FCITX5_AVAILABLE.store(false, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        },
    );

    // Callbacks: Minimize to System Tray (Windows) / Taskbar (Linux)
    let app_weak_min = app_weak.clone();
    #[cfg(windows)]
    let win_tray_for_min = win_tray_handle.clone();
    app.on_trigger_minimize(move || {
        if let Some(app) = app_weak_min.upgrade() {
            use i_slint_backend_winit::WinitWindowAccessor;
            let _ = app.window().with_winit_window(|winit_window| {
                #[cfg(windows)]
                {
                    winit_window.set_visible(false);
                }
                #[cfg(not(windows))]
                {
                    winit_window.set_minimized(true);
                }
            });
            #[cfg(windows)]
            if let Some(ref tray) = win_tray_for_min {
                tray.set_topbar_visible(false);
            }
        }
    });

    // Callbacks: Quit
    app.on_trigger_quit(move || {
        let _ = slint::quit_event_loop();
    });

    if is_standalone {
        standalone.run()?;
    } else {
        app.run()?;
    }
    Ok(())
}
