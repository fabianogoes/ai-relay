//! The application-local settings screen. It changes only the current run.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::language::Language;
use crate::theme;

use super::cards::{card, plain_title, put, seg};
use super::{FOOTER_SETTINGS, Freshness, MIN_HEIGHT, footer, header, layout};

/// The pending language choice and the effective language when settings opened.
pub struct SettingsScreen<'a> {
    pub workspace: &'a str,
    pub freshness: Freshness,
    pub language: Language,
    pub selected: Language,
}

impl Widget for &SettingsScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let places = layout(area);
        header(
            self.workspace,
            self.freshness,
            self.language,
            places.header,
            buf,
        );
        if area.height == 1 {
            return;
        }
        footer(&FOOTER_SETTINGS, self.language, places, buf);
        if area.height < MIN_HEIGHT || places.body.height < 4 {
            return;
        }

        let english = self.language == Language::En;
        let title = if english {
            "Settings"
        } else {
            "Configurações"
        };
        let language = if english { "Language" } else { "Idioma" };
        let inner = card(buf, places.body, &plain_title(title), &[], theme::DIM);
        put(buf, inner, 0, &[seg(language, theme::bold(theme::FG))]);

        for (row, option) in [Language::PtBr, Language::En].into_iter().enumerate() {
            let is_selected = option == self.selected;
            let is_current = option == self.language;
            let option_name = match (option, english) {
                (Language::En, _) => "English",
                (Language::PtBr, true) => "Portuguese (Brazil)",
                (Language::PtBr, false) => "Português (Brasil)",
            };
            let pointer = if is_selected { "▸ " } else { "  " };
            let style: Style = if is_selected {
                theme::bold(theme::BLUE)
            } else {
                theme::fg()
            };
            let mut line = vec![seg(format!("{pointer}{option_name}"), style)];
            if is_current {
                line.push(seg(
                    if english { " · current" } else { " · atual" },
                    theme::color(theme::META),
                ));
            }
            put(buf, inner, row + 1, &line);
        }
    }
}
