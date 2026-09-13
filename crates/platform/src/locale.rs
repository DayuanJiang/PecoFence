//! Windows display language (not the region/date-format preference).

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

pub fn ui_language() -> &'static str {
    // SAFETY: no arguments or pointers; available on every supported Windows version.
    let id = unsafe { GetUserDefaultUILanguage() };
    if matches!(id, 0x0404 | 0x0c04 | 0x1404) {
        return "zh-TW";
    }
    match id & 0x3ff {
        0x04 => "zh-CN",
        0x11 => "ja",
        0x12 => "ko",
        0x07 => "de",
        0x0c => "fr",
        0x0a => "es",
        0x16 => "pt-BR",
        0x19 => "ru",
        _ => "en",
    }
}
