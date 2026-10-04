//! COM GUIDs for Lekhani Text Services Framework (TSF) Input Processor

#[cfg(windows)]
use windows_sys::core::GUID;

#[cfg(windows)]
/// Unique CLSID for the Lekhani Text Input Processor COM Class
/// {7C149B90-D82A-4D7F-8BC2-D9A3E4E81A25}
pub const CLSID_LEKHANI_TIP: GUID = GUID::from_u128(0x7c149b90_d82a_4d7f_8bc2_d9a3e4e81a25);

#[cfg(windows)]
/// Unique Profile GUID for the Lekhani Bengali Keyboard Profile
/// {9B24D980-A4D2-475C-9A81-A6B4628DF801}
pub const GUID_LEKHANI_PROFILE: GUID = GUID::from_u128(0x9b24d980_a4d2_475c_9a81_a6b4628df801);

/// Primary Language ID for Bengali (Bangladesh)
pub const LANGID_BENGALI_BD: u16 = 0x0845;
/// Primary Language ID for Bengali (India)
pub const LANGID_BENGALI_IN: u16 = 0x0445;

#[cfg(windows)]
pub use windows_sys::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, CLSID_TF_ThreadMgr, GUID_PROP_ATTRIBUTE,
    GUID_PROP_COMPOSING, GUID_TFCAT_TIP_KEYBOARD,
};

#[cfg(windows)]
#[inline]
pub fn guid_eq(a: &GUID, b: &GUID) -> bool {
    a.data1 == b.data1 && a.data2 == b.data2 && a.data3 == b.data3 && a.data4 == b.data4
}
