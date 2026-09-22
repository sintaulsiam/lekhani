//! Lekhani Keyboard Layout Viewer UI Mapping

use crate::{StandaloneDialogWindow, TopBarWindow};
use lekhani_settings::LayoutManager;
use std::rc::Rc;

pub fn map_spec(
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

pub fn get_layout_rows(
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

pub fn update_viewer_ui(
    app: &TopBarWindow,
    layout_mgr: &LayoutManager,
    layout_name: &str,
    mode: i32,
) {
    let (r1, r2, r3, r4) = get_layout_rows(layout_mgr, layout_name, mode);
    app.set_viewer_row_1(
        Rc::new(slint::VecModel::from(
            r1.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    app.set_viewer_row_2(
        Rc::new(slint::VecModel::from(
            r2.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    app.set_viewer_row_3(
        Rc::new(slint::VecModel::from(
            r3.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    app.set_viewer_row_4(
        Rc::new(slint::VecModel::from(
            r4.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    app.set_viewer_selected_layout(layout_name.into());
    app.set_viewer_mode(mode);
}

pub fn update_standalone_viewer_ui(
    standalone: &StandaloneDialogWindow,
    layout_mgr: &LayoutManager,
    layout_name: &str,
    mode: i32,
) {
    let (r1, r2, r3, r4) = get_layout_rows(layout_mgr, layout_name, mode);
    standalone.set_viewer_row_1(
        Rc::new(slint::VecModel::from(
            r1.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    standalone.set_viewer_row_2(
        Rc::new(slint::VecModel::from(
            r2.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    standalone.set_viewer_row_3(
        Rc::new(slint::VecModel::from(
            r3.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    standalone.set_viewer_row_4(
        Rc::new(slint::VecModel::from(
            r4.into_iter()
                .map(slint::SharedString::from)
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    standalone.set_viewer_selected_layout(layout_name.into());
    standalone.set_viewer_mode(mode);
}
