//! Keyboard shortcuts for Peek: the text form `peek.customHotkey` stores ("Ctrl+Alt+Space"),
//! the keys a shortcut may end with, and the combinations PecoFence refuses to take.

/// Modifier bits, the values `RegisterHotKey` takes (`MOD_ALT`, `MOD_CONTROL`, `MOD_SHIFT`,
/// `MOD_WIN`).
pub const ALT: u32 = 1;
pub const CTRL: u32 = 2;
pub const SHIFT: u32 = 4;
pub const WIN: u32 = 8;

const SPACE: u32 = 0x20;
const TAB: u32 = 0x09;
const ESC: u32 = 0x1B;
const F4: u32 = 0x73;
const DELETE: u32 = 0x2E;

/// Modifiers in the order they are written.
const MODIFIERS: [(u32, &str); 4] = [(WIN, "Win"), (CTRL, "Ctrl"), (ALT, "Alt"), (SHIFT, "Shift")];

/// Keys with a name of their own (besides letters, digits, F1 to F24 and Num0 to Num9).
const NAMED: &[(u32, &str)] = &[
    (SPACE, "Space"),
    (0x0D, "Enter"),
    (TAB, "Tab"),
    (ESC, "Esc"),
    (0x08, "Backspace"),
    (0x2D, "Insert"),
    (DELETE, "Delete"),
    (0x24, "Home"),
    (0x23, "End"),
    (0x21, "PageUp"),
    (0x22, "PageDown"),
    (0x25, "Left"),
    (0x26, "Up"),
    (0x27, "Right"),
    (0x28, "Down"),
];

/// Other spellings [`Hotkey::parse`] accepts.
const ALIASES: &[(&str, &str)] = &[
    ("Escape", "Esc"),
    ("Return", "Enter"),
    ("Del", "Delete"),
    ("Ins", "Insert"),
    ("PgUp", "PageUp"),
    ("PgDn", "PageDown"),
    ("Control", "Ctrl"),
    ("Windows", "Win"),
];

/// Combinations that would take a common Windows function away from the user even though
/// `RegisterHotKey` may accept them. Win plus a single key is refused as a whole
/// ([`Problem::WinAlone`]).
const RESERVED: &[(u32, u32)] = &[
    // Input method on / off, the window menu.
    (CTRL, SPACE),
    (ALT, SPACE),
    // Switching windows, Start, Task Manager, closing a window, the security screen.
    (ALT, TAB),
    (ALT | SHIFT, TAB),
    (CTRL | ALT, TAB),
    (ALT, ESC),
    (CTRL, ESC),
    (CTRL | SHIFT, ESC),
    (ALT, F4),
    (CTRL | ALT, DELETE),
    // Select all, copy, paste, cut, undo, redo, save.
    (CTRL, b'A' as u32),
    (CTRL, b'C' as u32),
    (CTRL, b'V' as u32),
    (CTRL, b'X' as u32),
    (CTRL, b'Z' as u32),
    (CTRL, b'Y' as u32),
    (CTRL, b'S' as u32),
];

/// A modifier set plus a virtual-key code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub mods: u32,
    pub vk: u32,
}

/// Why a combination cannot be the Peek shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// Not of the form `Mod+...+Key`, or a modifier name is unknown.
    Malformed,
    /// The last key is not one of the keys a shortcut may end with.
    UnsupportedKey,
    /// Neither Ctrl, Alt nor Win is held.
    NoModifier,
    /// Win plus a single key: Windows keeps those for itself.
    WinAlone,
    /// A common Windows shortcut (see `RESERVED`).
    Reserved,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Malformed => "write it as modifiers and a key joined by +, such as Ctrl+Alt+K",
            Self::UnsupportedKey => {
                "the key must be a letter, a digit, F1 to F24, Num0 to Num9, Space, Enter, Tab, Esc, Backspace, Insert, Delete, Home, End, PageUp, PageDown or an arrow key"
            }
            Self::NoModifier => "a shortcut needs Ctrl, Alt or Win",
            Self::WinAlone => "Windows keeps Win plus a single key for itself; add Ctrl, Alt or Shift",
            Self::Reserved => "it is a common Windows shortcut",
        })
    }
}

/// The name of a key a shortcut may end with (`None` for any other key).
pub fn key_name(vk: u32) -> Option<String> {
    match vk {
        0x41..=0x5A | 0x30..=0x39 => Some(char::from(vk as u8).to_string()),
        0x70..=0x87 => Some(format!("F{}", vk - 0x6F)),
        0x60..=0x69 => Some(format!("Num{}", vk - 0x60)),
        _ => NAMED
            .iter()
            .find(|(k, _)| *k == vk)
            .map(|(_, n)| n.to_string()),
    }
}

fn key_code(name: &str) -> Option<u32> {
    let upper = name.to_ascii_uppercase();
    if let [c] = upper.as_bytes()
        && c.is_ascii_alphanumeric()
    {
        return Some(u32::from(*c));
    }
    let number = |prefix: &str, first: u32, count: u32| {
        upper
            .strip_prefix(prefix)
            .filter(|n| !n.starts_with('0') || n.len() == 1)
            .and_then(|n| n.parse::<u32>().ok())
            .filter(|n| (first..first + count).contains(n))
    };
    if let Some(n) = number("F", 1, 24) {
        return Some(0x6F + n);
    }
    if let Some(n) = number("NUM", 0, 10) {
        return Some(0x60 + n);
    }
    NAMED
        .iter()
        .find(|(_, n)| n.eq_ignore_ascii_case(name))
        .map(|(k, _)| *k)
}

/// `name` with an alias replaced by the name PecoFence writes.
fn canonical(name: &str) -> &str {
    ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        .map_or(name, |(_, n)| n)
}

impl Hotkey {
    pub const fn new(mods: u32, vk: u32) -> Self {
        Self { mods, vk }
    }

    /// Reads `Ctrl+Alt+Space` (any case, spaces around `+` allowed) without judging the
    /// combination; see [`Self::check`].
    pub fn parse(text: &str) -> Result<Self, Problem> {
        let parts: Vec<&str> = text.split('+').map(str::trim).collect();
        let (key, modifiers) = parts.split_last().ok_or(Problem::Malformed)?;
        if key.is_empty() {
            return Err(Problem::Malformed);
        }
        let mut mods = 0;
        for m in modifiers {
            let m = canonical(m);
            let (bit, _) = MODIFIERS
                .iter()
                .find(|(_, n)| n.eq_ignore_ascii_case(m))
                .ok_or(Problem::Malformed)?;
            mods |= bit;
        }
        let vk = key_code(canonical(key)).ok_or(Problem::UnsupportedKey)?;
        Ok(Self { mods, vk })
    }

    /// Whether this combination may be the Peek shortcut: a supported key, at least Ctrl, Alt or
    /// Win, not Win plus the key alone and not a common Windows shortcut. Whether another
    /// program holds it only `RegisterHotKey` can tell.
    pub fn check(self) -> Result<Self, Problem> {
        if self.mods & !(ALT | CTRL | SHIFT | WIN) != 0 || key_name(self.vk).is_none() {
            return Err(Problem::UnsupportedKey);
        }
        if self.mods & (CTRL | ALT | WIN) == 0 {
            return Err(Problem::NoModifier);
        }
        if self.mods == WIN {
            return Err(Problem::WinAlone);
        }
        if RESERVED.contains(&(self.mods, self.vk)) {
            return Err(Problem::Reserved);
        }
        Ok(self)
    }

    /// The modifier names and the key name, in writing order (`["Ctrl", "Alt", "Space"]`).
    pub fn parts(self) -> Vec<String> {
        MODIFIERS
            .iter()
            .filter(|(bit, _)| self.mods & bit != 0)
            .map(|(_, n)| n.to_string())
            .chain(Some(
                key_name(self.vk).unwrap_or_else(|| format!("0x{:02X}", self.vk)),
            ))
            .collect()
    }
}

/// `Ctrl+Alt+Space`, the form [`Hotkey::parse`] reads back.
impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.parts().join("+"))
    }
}

pub const CTRL_ALT_SPACE: Hotkey = Hotkey::new(CTRL | ALT, SPACE);

/// The quick choices on the settings page, which are also the fallbacks when the chosen
/// shortcut is taken. Windows 10 and 11 keep Win+Space and Win+Shift+Space for the input
/// switcher even with a single keyboard layout; these three are free on both.
pub const PRESETS: [Hotkey; 3] = [
    CTRL_ALT_SPACE,
    Hotkey::new(WIN | ALT, SPACE),
    Hotkey::new(CTRL | SHIFT, SPACE),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trips() {
        for text in [
            "Ctrl+Alt+Space",
            "Win+Alt+Space",
            "Win+Ctrl+Alt+Shift+K",
            "Ctrl+Shift+F12",
            "Alt+Num7",
            "Ctrl+Alt+PageDown",
            "Win+Shift+0",
        ] {
            let h = Hotkey::parse(text).unwrap();
            assert_eq!(h.to_string(), text);
        }
        assert_eq!(CTRL_ALT_SPACE.to_string(), "Ctrl+Alt+Space");
    }

    #[test]
    fn parse_is_lenient_about_case_spaces_aliases_and_order() {
        let h = Hotkey::new(CTRL | ALT, b'K' as u32);
        assert_eq!(Hotkey::parse("ctrl + alt + k"), Ok(h));
        assert_eq!(Hotkey::parse("Alt+Control+K"), Ok(h));
        assert_eq!(
            Hotkey::parse("Windows+Shift+Escape"),
            Ok(Hotkey::new(WIN | SHIFT, ESC))
        );
        assert_eq!(
            Hotkey::parse("Ctrl+Alt+Del"),
            Ok(Hotkey::new(CTRL | ALT, DELETE))
        );
        assert_eq!(Hotkey::parse("Ctrl+f1"), Ok(Hotkey::new(CTRL, 0x70)));
    }

    #[test]
    fn parse_rejects_what_it_cannot_read() {
        assert_eq!(Hotkey::parse(""), Err(Problem::Malformed));
        assert_eq!(Hotkey::parse("Ctrl+"), Err(Problem::Malformed));
        assert_eq!(Hotkey::parse("Hyper+K"), Err(Problem::Malformed));
        assert_eq!(Hotkey::parse("Ctrl+Alt+Ä"), Err(Problem::UnsupportedKey));
        assert_eq!(Hotkey::parse("Ctrl+F25"), Err(Problem::UnsupportedKey));
        assert_eq!(Hotkey::parse("Ctrl+F01"), Err(Problem::UnsupportedKey));
        assert_eq!(Hotkey::parse("Ctrl+Num10"), Err(Problem::UnsupportedKey));
        assert_eq!(Hotkey::parse("Ctrl+Alt"), Err(Problem::UnsupportedKey));
    }

    #[test]
    fn check_needs_ctrl_alt_or_win() {
        assert_eq!(
            Hotkey::new(SHIFT, b'K' as u32).check(),
            Err(Problem::NoModifier)
        );
        assert_eq!(Hotkey::new(0, 0x70).check(), Err(Problem::NoModifier));
        assert!(Hotkey::new(CTRL | ALT, b'K' as u32).check().is_ok());
        assert!(Hotkey::new(ALT | SHIFT, 0x70).check().is_ok());
    }

    #[test]
    fn check_refuses_windows_shortcuts() {
        for text in [
            "Ctrl+Space",
            "Alt+Space",
            "Alt+Tab",
            "Ctrl+Alt+Delete",
            "Ctrl+Shift+Esc",
            "Alt+F4",
            "Ctrl+C",
            "Ctrl+V",
        ] {
            let h = Hotkey::parse(text).unwrap();
            assert_eq!(h.check(), Err(Problem::Reserved), "{text}");
        }
        for text in ["Win+L", "Win+Space", "Win+F12", "Win+D"] {
            let h = Hotkey::parse(text).unwrap();
            assert_eq!(h.check(), Err(Problem::WinAlone), "{text}");
        }
        // With another modifier Win is fine (Windows refuses the ones it uses at registration).
        assert!(Hotkey::parse("Win+Shift+Space").unwrap().check().is_ok());
        assert!(Hotkey::parse("Ctrl+Shift+C").unwrap().check().is_ok());
    }

    #[test]
    fn check_refuses_keys_the_page_cannot_name() {
        // VK_OEM_1 (; on US keyboards) and VK_LWIN.
        assert_eq!(
            Hotkey::new(CTRL | ALT, 0xBA).check(),
            Err(Problem::UnsupportedKey)
        );
        assert_eq!(
            Hotkey::new(CTRL | ALT, 0x5B).check(),
            Err(Problem::UnsupportedKey)
        );
        assert_eq!(
            Hotkey::new(CTRL | 0x4000, SPACE).check(),
            Err(Problem::UnsupportedKey)
        );
    }

    #[test]
    fn presets_pass_the_check() {
        for p in PRESETS {
            assert_eq!(p.check(), Ok(p));
        }
    }
}
