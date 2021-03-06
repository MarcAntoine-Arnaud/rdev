use core_foundation::base::{CFRelease, OSStatus};
use core_foundation::string::UniChar;
use core_foundation_sys::data::{CFDataGetBytePtr, CFDataRef};
use core_graphics::event::CGEventFlags;
use std::convert::TryInto;
use std::ffi::c_void;
use std::os::raw::c_uint;

type TISInputSourceRef = *mut c_void;
type ModifierState = u32;
type UniCharCount = usize;

type OptionBits = c_uint;
#[allow(non_upper_case_globals)]
static kUCKeyTranslateDeadKeysBit: OptionBits = 1 << 31;
#[allow(non_upper_case_globals)]
static kUCKeyActionDown: u16 = 0;
#[allow(non_upper_case_globals)]
static NSEventModifierFlagCapsLock: u64 = 1 << 16;
#[allow(non_upper_case_globals)]
static NSEventModifierFlagShift: u64 = 1 << 17;
#[allow(non_upper_case_globals)]
static NSEventModifierFlagControl: u64 = 1 << 18;
#[allow(non_upper_case_globals)]
static NSEventModifierFlagOption: u64 = 1 << 19;
#[allow(non_upper_case_globals)]
static NSEventModifierFlagCommand: u64 = 1 << 20;

const BUF_LEN: usize = 4;

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
    fn TISGetInputSourceProperty(source: TISInputSourceRef, property: *mut c_void) -> CFDataRef;
    fn UCKeyTranslate(
        layout: *const u8,
        code: u16,
        key_action: u16,
        modifier_state: u32,
        keyboard_type: u32,
        key_translate_options: OptionBits,
        dead_key_state: *mut u32,
        max_length: UniCharCount,
        actual_length: *mut UniCharCount,
        unicode_string: *mut [UniChar; BUF_LEN],
    ) -> OSStatus;
    fn LMGetKbdType() -> u32;
    static kTISPropertyUnicodeKeyLayoutData: *mut c_void;

}

pub struct KeyboardState {
    pub dead_state: u32,
}
impl KeyboardState {
    pub unsafe fn create_string_for_key(
        &mut self,
        code: u32,
        flags: CGEventFlags,
    ) -> Option<String> {
        let modifier_state = flags_to_state(flags.bits());
        let keyboard = TISCopyCurrentKeyboardInputSource();
        let layout = TISGetInputSourceProperty(keyboard, kTISPropertyUnicodeKeyLayoutData);
        let layout_ptr = CFDataGetBytePtr(layout);

        let mut buff = [0 as UniChar; BUF_LEN];
        let kb_type = LMGetKbdType();
        let mut length = 0;
        let _retval = UCKeyTranslate(
            layout_ptr,
            code.try_into().ok()?,
            kUCKeyActionDown,
            modifier_state,
            kb_type,
            kUCKeyTranslateDeadKeysBit,
            &mut self.dead_state,                 // deadKeyState
            BUF_LEN,                              // max string length
            &mut length as *mut UniCharCount,     // actual string length
            &mut buff as *mut [UniChar; BUF_LEN], // unicode string
        );
        CFRelease(keyboard);

        String::from_utf16(&buff[..length]).ok()
    }
}

#[allow(clippy::identity_op)]
pub unsafe fn flags_to_state(flags: u64) -> ModifierState {
    let has_alt = flags & NSEventModifierFlagOption;
    let has_caps_lock = flags & NSEventModifierFlagCapsLock;
    let has_control = flags & NSEventModifierFlagControl;
    let has_shift = flags & NSEventModifierFlagShift;
    let has_meta = flags & NSEventModifierFlagCommand;
    let mut modifier = 0;
    if has_alt != 0 {
        modifier += 1 << 3;
    }
    if has_caps_lock != 0 {
        modifier += 1 << 1;
    }
    if has_control != 0 {
        modifier += 1 << 4;
    }
    if has_shift != 0 {
        modifier += 1 << 1;
    }
    if has_meta != 0 {
        modifier += 1 << 0;
    }
    modifier
}
