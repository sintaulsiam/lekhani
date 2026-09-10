//! IBus / X11 Keyval to Lekhani Virtual Keycode Mapping

use hashbrown::HashMap;
use lekhani_core::keycodes::*;

pub struct KeycodeMapper {
    map: HashMap<u32, u16>,
}

impl Default for KeycodeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl KeycodeMapper {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Alphanumeric Keys
        map.insert(0x0060, VC_GRAVE);
        map.insert(0xfe50, VC_GRAVE);
        map.insert(0x007e, VC_TILDE);

        map.insert(0x0030, VC_0);
        map.insert(0x0031, VC_1);
        map.insert(0x0032, VC_2);
        map.insert(0x0033, VC_3);
        map.insert(0x0034, VC_4);
        map.insert(0x0035, VC_5);
        map.insert(0x0036, VC_6);
        map.insert(0x0037, VC_7);
        map.insert(0x0038, VC_8);
        map.insert(0x0039, VC_9);

        map.insert(0x0029, VC_PAREN_RIGHT);
        map.insert(0x0021, VC_EXCLAIM);
        map.insert(0x0040, VC_AT);
        map.insert(0x0023, VC_HASH);
        map.insert(0x0024, VC_DOLLAR);
        map.insert(0x0025, VC_PERCENT);
        map.insert(0x005e, VC_CIRCUM);
        map.insert(0x0026, VC_AMPERSAND);
        map.insert(0x002a, VC_ASTERISK);
        map.insert(0x0028, VC_PAREN_LEFT);

        map.insert(0x002d, VC_MINUS);
        map.insert(0x005f, VC_UNDERSCORE);
        map.insert(0x003d, VC_EQUALS);
        map.insert(0x002b, VC_PLUS);

        // Alphabet Lowercase
        map.insert(0x0061, VC_A);
        map.insert(0x0062, VC_B);
        map.insert(0x0063, VC_C);
        map.insert(0x0064, VC_D);
        map.insert(0x0065, VC_E);
        map.insert(0x0066, VC_F);
        map.insert(0x0067, VC_G);
        map.insert(0x0068, VC_H);
        map.insert(0x0069, VC_I);
        map.insert(0x006a, VC_J);
        map.insert(0x006b, VC_K);
        map.insert(0x006c, VC_L);
        map.insert(0x006d, VC_M);
        map.insert(0x006e, VC_N);
        map.insert(0x006f, VC_O);
        map.insert(0x0070, VC_P);
        map.insert(0x0071, VC_Q);
        map.insert(0x0072, VC_R);
        map.insert(0x0073, VC_S);
        map.insert(0x0074, VC_T);
        map.insert(0x0075, VC_U);
        map.insert(0x0076, VC_V);
        map.insert(0x0077, VC_W);
        map.insert(0x0078, VC_X);
        map.insert(0x0079, VC_Y);
        map.insert(0x007a, VC_Z);

        // Alphabet Uppercase
        map.insert(0x0041, VC_A_SHIFT);
        map.insert(0x0042, VC_B_SHIFT);
        map.insert(0x0043, VC_C_SHIFT);
        map.insert(0x0044, VC_D_SHIFT);
        map.insert(0x0045, VC_E_SHIFT);
        map.insert(0x0046, VC_F_SHIFT);
        map.insert(0x0047, VC_G_SHIFT);
        map.insert(0x0048, VC_H_SHIFT);
        map.insert(0x0049, VC_I_SHIFT);
        map.insert(0x004a, VC_J_SHIFT);
        map.insert(0x004b, VC_K_SHIFT);
        map.insert(0x004c, VC_L_SHIFT);
        map.insert(0x004d, VC_M_SHIFT);
        map.insert(0x004e, VC_N_SHIFT);
        map.insert(0x004f, VC_O_SHIFT);
        map.insert(0x0050, VC_P_SHIFT);
        map.insert(0x0051, VC_Q_SHIFT);
        map.insert(0x0052, VC_R_SHIFT);
        map.insert(0x0053, VC_S_SHIFT);
        map.insert(0x0054, VC_T_SHIFT);
        map.insert(0x0055, VC_U_SHIFT);
        map.insert(0x0056, VC_V_SHIFT);
        map.insert(0x0057, VC_W_SHIFT);
        map.insert(0x0058, VC_X_SHIFT);
        map.insert(0x0059, VC_Y_SHIFT);
        map.insert(0x005a, VC_Z_SHIFT);

        // Punctuation
        map.insert(0x005b, VC_BRACKET_LEFT);
        map.insert(0x007b, VC_BRACE_LEFT);
        map.insert(0x005d, VC_BRACKET_RIGHT);
        map.insert(0x007d, VC_BRACE_RIGHT);
        map.insert(0x005c, VC_BACK_SLASH);
        map.insert(0x007c, VC_BAR);
        map.insert(0x002f, VC_SLASH);
        map.insert(0x003f, VC_QUESTION);
        map.insert(0x003b, VC_SEMICOLON);
        map.insert(0x003a, VC_COLON);
        map.insert(0x002c, VC_COMMA);
        map.insert(0x003c, VC_LESS);
        map.insert(0x002e, VC_PERIOD);
        map.insert(0x003e, VC_GREATER);
        map.insert(0x0027, VC_APOSTROPHE);
        map.insert(0x0022, VC_QUOTE);

        // NumPad
        map.insert(0xffaf, VC_KP_DIVIDE);
        map.insert(0xffaa, VC_KP_MULTIPLY);
        map.insert(0xffad, VC_KP_SUBTRACT);
        map.insert(0xffab, VC_KP_ADD);
        map.insert(0xffae, VC_KP_DECIMAL);
        map.insert(0xff8d, VC_KP_ENTER);
        map.insert(0xffbd, VC_KP_EQUALS);

        map.insert(0xffb0, VC_KP_0);
        map.insert(0xffb1, VC_KP_1);
        map.insert(0xffb2, VC_KP_2);
        map.insert(0xffb3, VC_KP_3);
        map.insert(0xffb4, VC_KP_4);
        map.insert(0xffb5, VC_KP_5);
        map.insert(0xffb6, VC_KP_6);
        map.insert(0xffb7, VC_KP_7);
        map.insert(0xffb8, VC_KP_8);
        map.insert(0xffb9, VC_KP_9);

        Self { map }
    }

    pub fn map_keyval(&self, keyval: u32) -> u16 {
        self.map.get(&keyval).copied().unwrap_or(VC_UNKNOWN)
    }
}
