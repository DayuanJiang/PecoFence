//! A notice card in the fence look: title, one message, an accent action button (with an
//! optional plain button beside it, e.g. 「不用」) and a close glyph on an opaque card (the
//! app's `notice.rs` owns the window). Sizes follow the Windows 11
//! toast: 360 DIPs wide, Body Strong 14 title, Body 14 message, a 32 DIP button.

use crate::theme::{Theme, font_icons, font_text, with_alpha};
use windows_canvas::{
    ColorF, DrawingSession, FontWeight, ParagraphAlignment, Rect, RoundedRect, TextAlignment,
    TextFormat, TextLayout, WordWrapping,
};
use windows_core::Result;

pub const WIDTH: f32 = 360.0;
const PAD: f32 = 16.0;
const CLOSE: f32 = 32.0;
const BUTTON_H: f32 = 32.0;
const BUTTON_MIN_W: f32 = 120.0;
/// Between the plain and the accent button (WinUI's ContentDialog button spacing).
const BUTTON_GAP: f32 = 8.0;
/// ChromeClose in Segoe Fluent Icons.
const CLOSE_GLYPH: &str = "\u{E8BB}";

/// What the pointer is over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoticeHit {
    #[default]
    None,
    /// The card itself, which acts like the button (a toast's body opens it too) unless the
    /// card has a second button: then the body does nothing.
    Card,
    Button,
    /// The plain button left of the accent one ([`NoticeCard::with_secondary`]).
    Secondary,
    Close,
}

/// Rectangles in DIPs, origin at the card's top-left.
#[derive(Clone, Copy, Debug)]
pub struct NoticeLayout {
    pub height: f32,
    title: Rect,
    body: Rect,
    button: Rect,
    secondary: Option<Rect>,
    close: Rect,
}

impl NoticeLayout {
    /// The card has a second button, so a click on its body does nothing.
    pub fn has_secondary(&self) -> bool {
        self.secondary.is_some()
    }

    pub fn hit(&self, x: f32, y: f32) -> NoticeHit {
        let inside = |r: &Rect| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
        if !(0.0..WIDTH).contains(&x) || !(0.0..self.height).contains(&y) {
            NoticeHit::None
        } else if inside(&self.close) {
            NoticeHit::Close
        } else if inside(&self.button) {
            NoticeHit::Button
        } else if self.secondary.as_ref().is_some_and(inside) {
            NoticeHit::Secondary
        } else {
            NoticeHit::Card
        }
    }
}

pub struct NoticeCard {
    title: String,
    body: String,
    action: String,
    secondary: Option<String>,
    /// The card is at least this tall (DIPs): the room goes between the text and the buttons.
    min_height: f32,
    title_format: TextFormat,
    body_format: TextFormat,
    button_format: TextFormat,
    glyph_format: TextFormat,
}

impl NoticeCard {
    /// `locale` picks the fonts for Han, kana and Hangul (the text's language, e.g. `zh-CN`).
    pub fn new(title: &str, body: &str, action: &str, locale: &str) -> Result<Self> {
        let wrapping = |f: TextFormat| {
            f.with_alignment(TextAlignment::Leading)
                .with_paragraph_alignment(ParagraphAlignment::Top)
                .with_word_wrapping(WordWrapping::Wrap)
        };
        let centred = |f: TextFormat| {
            f.with_alignment(TextAlignment::Center)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap)
        };
        Ok(Self {
            title: title.to_string(),
            body: body.to_string(),
            action: action.to_string(),
            secondary: None,
            min_height: 0.0,
            title_format: wrapping(TextFormat::with_locale(
                font_text(),
                14.0,
                FontWeight(600),
                locale,
            )?),
            body_format: wrapping(TextFormat::with_locale(
                font_text(),
                14.0,
                FontWeight::NORMAL,
                locale,
            )?),
            button_format: centred(TextFormat::with_locale(
                font_text(),
                14.0,
                FontWeight::NORMAL,
                locale,
            )?),
            glyph_format: centred(TextFormat::with_locale(
                font_icons(),
                10.0,
                FontWeight::NORMAL,
                locale,
            )?),
        })
    }

    /// Adds a plain button labelled `text` left of the action button (「不用」 beside
    /// 「试一试」).
    pub fn with_secondary(mut self, text: &str) -> Self {
        self.secondary = Some(text.to_string());
        self
    }

    /// Makes the card at least `height` DIPs tall (a row of cards of one size: the tips). The
    /// buttons stay at the bottom edge.
    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    pub fn layout(&self) -> Result<NoticeLayout> {
        let text_w = WIDTH - PAD - CLOSE - 8.0;
        let title_h = TextLayout::new(&self.title, &self.title_format, text_w, 1000.0)?
            .metrics()
            .height;
        let title = Rect::new(PAD, PAD - 2.0, PAD + text_w, PAD - 2.0 + title_h);
        let body_w = WIDTH - 2.0 * PAD;
        let body_h = TextLayout::new(&self.body, &self.body_format, body_w, 1000.0)?
            .metrics()
            .height;
        let body = Rect::new(
            PAD,
            title.bottom + 4.0,
            PAD + body_w,
            title.bottom + 4.0 + body_h,
        );
        let label_w = TextLayout::new(&self.action, &self.button_format, 1000.0, 100.0)?
            .metrics()
            .width;
        let button_w = (label_w + 24.0).max(BUTTON_MIN_W).min(body_w);
        let button_top = (body.bottom + 16.0).max(self.min_height - PAD - BUTTON_H);
        let button = Rect::new(
            WIDTH - PAD - button_w,
            button_top,
            WIDTH - PAD,
            button_top + BUTTON_H,
        );
        let secondary = match &self.secondary {
            Some(text) => {
                let label_w = TextLayout::new(text, &self.button_format, 1000.0, 100.0)?
                    .metrics()
                    .width;
                let w = (label_w + 24.0)
                    .max(BUTTON_MIN_W)
                    .min(body_w - button_w - BUTTON_GAP);
                let right = button.left - BUTTON_GAP;
                Some(Rect::new(
                    right - w,
                    button_top,
                    right,
                    button_top + BUTTON_H,
                ))
            }
            None => None,
        };
        let close = Rect::new(WIDTH - 8.0 - CLOSE, 8.0, WIDTH - 8.0, 8.0 + CLOSE);
        Ok(NoticeLayout {
            height: button.bottom + PAD,
            title,
            body,
            button,
            secondary,
            close,
        })
    }

    pub fn draw(
        &self,
        s: &DrawingSession<'_>,
        layout: &NoticeLayout,
        theme: &Theme,
        hover: NoticeHit,
    ) -> Result<()> {
        let card = Rect::new(0.0, 0.0, WIDTH, layout.height);
        s.clear(ColorF {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
        s.fill_rounded_rect(
            &RoundedRect::uniform(card, theme.corner_radius),
            &s.create_solid_brush(with_alpha(theme.solid_fill, 1.0))?,
        );
        if hover == NoticeHit::Card && layout.secondary.is_none() {
            s.fill_rounded_rect(
                &RoundedRect::uniform(card, theme.corner_radius),
                &s.create_solid_brush(theme.subtle_hover)?,
            );
        }
        let inset = Rect::new(0.5, 0.5, WIDTH - 0.5, layout.height - 0.5);
        s.draw_rounded_rect(
            &RoundedRect::uniform(inset, theme.corner_radius),
            &s.create_solid_brush(theme.stroke)?,
            1.0,
        );

        s.draw_text(
            &self.title,
            &self.title_format,
            &layout.title,
            &s.create_solid_brush(theme.text_primary)?,
        );
        s.draw_text(
            &self.body,
            &self.body_format,
            &layout.body,
            &s.create_solid_brush(theme.text_secondary)?,
        );

        // Accent button with the text colour WinUI puts on it (dark on the light dark-mode
        // accent, white on the deep light-mode one).
        let accent = theme.accent;
        let on_accent = if 0.2126 * accent.r + 0.7152 * accent.g + 0.0722 * accent.b > 0.5 {
            ColorF {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            ColorF {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
        };
        let fill = if hover == NoticeHit::Button {
            with_alpha(accent, 0.9)
        } else {
            accent
        };
        s.fill_rounded_rect(
            &RoundedRect::uniform(layout.button, 4.0),
            &s.create_solid_brush(fill)?,
        );
        s.draw_text(
            &self.action,
            &self.button_format,
            &layout.button,
            &s.create_solid_brush(on_accent)?,
        );

        // The plain button: WinUI's standard button (control fill, hairline stroke).
        if let (Some(text), Some(rect)) = (&self.secondary, &layout.secondary) {
            let shape = RoundedRect::uniform(*rect, 4.0);
            s.fill_rounded_rect(&shape, &s.create_solid_brush(theme.layer_fill)?);
            if hover == NoticeHit::Secondary {
                s.fill_rounded_rect(&shape, &s.create_solid_brush(theme.subtle_hover)?);
            }
            let inset = Rect::new(
                rect.left + 0.5,
                rect.top + 0.5,
                rect.right - 0.5,
                rect.bottom - 0.5,
            );
            s.draw_rounded_rect(
                &RoundedRect::uniform(inset, 4.0),
                &s.create_solid_brush(theme.stroke)?,
                1.0,
            );
            s.draw_text(
                text,
                &self.button_format,
                rect,
                &s.create_solid_brush(theme.text_primary)?,
            );
        }

        if hover == NoticeHit::Close {
            s.fill_rounded_rect(
                &RoundedRect::uniform(layout.close, 4.0),
                &s.create_solid_brush(theme.subtle_hover)?,
            );
        }
        s.draw_text(
            CLOSE_GLYPH,
            &self.glyph_format,
            &layout.close,
            &s.create_solid_brush(theme.text_secondary)?,
        );
        Ok(())
    }
}
