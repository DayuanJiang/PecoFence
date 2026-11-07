//! The two small buttons a rectangle dragged on the empty desktop offers at its bottom-right
//! corner (the app's `marquee_offer.rs` owns the window): 「在此显示文件夹…」 in the subtle
//! style, then 「＋ 新建栅栏」 in the accent colour at the corner. Fluent buttons have 4 DIP
//! corners, Liquid Glass ones are pills.

use crate::theme::{Theme, font_icons, font_text, with_alpha};
use windows_canvas::{
    ColorF, DrawingSession, FontWeight, ParagraphAlignment, Rect, RoundedRect, TextAlignment,
    TextFormat, TextLayout, WordWrapping,
};
use windows_core::Result;

pub const BUTTON_H: f32 = 32.0;
/// Between the two buttons.
const GAP: f32 = 6.0;
const PAD_X: f32 = 12.0;
/// The glyph before each label and the space after it.
const GLYPH_W: f32 = 22.0;
/// Add and Folder in Segoe Fluent Icons.
const ADD_GLYPH: &str = "\u{E710}";
const FOLDER_GLYPH: &str = "\u{E8B7}";

/// What the pointer is over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OfferHit {
    #[default]
    None,
    /// 在此显示文件夹…
    Folder,
    /// 新建栅栏
    NewFence,
}

/// Rectangles in DIPs, origin at the window's top-left.
#[derive(Clone, Copy, Debug)]
pub struct OfferLayout {
    pub width: f32,
    pub height: f32,
    folder: Rect,
    new_fence: Rect,
}

impl OfferLayout {
    pub fn hit(&self, x: f32, y: f32) -> OfferHit {
        let inside = |r: &Rect| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
        if inside(&self.new_fence) {
            OfferHit::NewFence
        } else if inside(&self.folder) {
            OfferHit::Folder
        } else {
            OfferHit::None
        }
    }
}

pub struct OfferButtons {
    new_fence: String,
    folder: String,
    text_format: TextFormat,
    glyph_format: TextFormat,
}

impl OfferButtons {
    /// `locale` picks the fonts for Han, kana and Hangul (the labels' language).
    pub fn new(new_fence: &str, folder: &str, locale: &str) -> Result<Self> {
        let centred = |f: TextFormat| {
            f.with_alignment(TextAlignment::Leading)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap)
        };
        Ok(Self {
            new_fence: new_fence.to_string(),
            folder: folder.to_string(),
            text_format: centred(TextFormat::with_locale(
                font_text(),
                14.0,
                FontWeight::NORMAL,
                locale,
            )?),
            glyph_format: centred(TextFormat::with_locale(
                font_icons(),
                14.0,
                FontWeight::NORMAL,
                locale,
            )?),
        })
    }

    pub fn layout(&self) -> Result<OfferLayout> {
        let width_of = |label: &str| -> Result<f32> {
            let w = TextLayout::new(label, &self.text_format, 1000.0, 100.0)?
                .metrics()
                .width;
            Ok((PAD_X + GLYPH_W + w + PAD_X).ceil())
        };
        let folder_w = width_of(&self.folder)?;
        let new_w = width_of(&self.new_fence)?;
        let folder = Rect::new(0.0, 0.0, folder_w, BUTTON_H);
        let new_fence = Rect::new(folder_w + GAP, 0.0, folder_w + GAP + new_w, BUTTON_H);
        Ok(OfferLayout {
            width: new_fence.right,
            height: BUTTON_H,
            folder,
            new_fence,
        })
    }

    pub fn draw(
        &self,
        s: &DrawingSession<'_>,
        layout: &OfferLayout,
        theme: &Theme,
        hover: OfferHit,
        pressed: OfferHit,
    ) -> Result<()> {
        s.clear(ColorF {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
        let radius = if theme.liquid_glass {
            BUTTON_H * 0.5
        } else {
            4.0
        };
        // 在此显示文件夹…: the opaque card colour with a hairline, like a standard button.
        let r = layout.folder;
        s.fill_rounded_rect(
            &RoundedRect::uniform(r, radius),
            &s.create_solid_brush(with_alpha(theme.solid_fill, 1.0))?,
        );
        if pressed == OfferHit::Folder {
            s.fill_rounded_rect(
                &RoundedRect::uniform(r, radius),
                &s.create_solid_brush(theme.subtle_pressed)?,
            );
        } else if hover == OfferHit::Folder {
            s.fill_rounded_rect(
                &RoundedRect::uniform(r, radius),
                &s.create_solid_brush(theme.subtle_hover)?,
            );
        }
        let inset = Rect::new(r.left + 0.5, r.top + 0.5, r.right - 0.5, r.bottom - 0.5);
        s.draw_rounded_rect(
            &RoundedRect::uniform(inset, radius - 0.5),
            &s.create_solid_brush(theme.stroke)?,
            1.0,
        );
        let ink = s.create_solid_brush(theme.text_primary)?;
        self.draw_label(s, r, FOLDER_GLYPH, &self.folder, &ink)?;

        // 新建栅栏: accent, with the text colour WinUI puts on it (as the notice's button).
        let r = layout.new_fence;
        let accent = theme.accent;
        let fill = match (pressed, hover) {
            (OfferHit::NewFence, _) => with_alpha(accent, 0.8),
            (_, OfferHit::NewFence) => with_alpha(accent, 0.9),
            _ => accent,
        };
        s.fill_rounded_rect(
            &RoundedRect::uniform(r, radius),
            &s.create_solid_brush(fill)?,
        );
        let dark_text = 0.2126 * accent.r + 0.7152 * accent.g + 0.0722 * accent.b > 0.5;
        let c = if dark_text { 0.0 } else { 1.0 };
        let on_accent = s.create_solid_brush(ColorF {
            r: c,
            g: c,
            b: c,
            a: 1.0,
        })?;
        self.draw_label(s, r, ADD_GLYPH, &self.new_fence, &on_accent)?;
        Ok(())
    }

    fn draw_label(
        &self,
        s: &DrawingSession<'_>,
        r: Rect,
        glyph: &str,
        label: &str,
        brush: &windows_canvas::Brush,
    ) -> Result<()> {
        let glyph_r = Rect::new(r.left + PAD_X, r.top, r.left + PAD_X + GLYPH_W, r.bottom);
        s.draw_text(glyph, &self.glyph_format, &glyph_r, brush);
        let text_r = Rect::new(glyph_r.right, r.top, r.right, r.bottom);
        s.draw_text(label, &self.text_format, &text_r, brush);
        Ok(())
    }
}
