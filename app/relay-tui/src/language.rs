//! Language selection for the interface and its command-line messages.

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    PtBr,
}

impl Language {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "en" => Some(Self::En),
            "pt-BR" => Some(Self::PtBr),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Locale {
    pub lc_all: Option<String>,
    pub lc_messages: Option<String>,
    pub lang: Option<String>,
}

impl Locale {
    pub fn from_env() -> Self {
        Self {
            lc_all: env::var("LC_ALL").ok(),
            lc_messages: env::var("LC_MESSAGES").ok(),
            lang: env::var("LANG").ok(),
        }
    }

    fn language(&self) -> Option<Language> {
        [&self.lc_all, &self.lc_messages, &self.lang]
            .into_iter()
            .filter_map(Option::as_deref)
            .find(|value| !value.is_empty())
            .map(|value| {
                if value.to_ascii_lowercase().starts_with("pt") {
                    Language::PtBr
                } else {
                    Language::En
                }
            })
    }
}

pub fn parse_settings(contents: &str) -> Option<Language> {
    let mut in_fence = false;
    let mut found = None;
    let mut count = 0;
    for line in contents.lines() {
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(value) = line.strip_prefix("- Language: ") {
            count += 1;
            found = Language::parse(value);
        }
    }
    (count == 1).then_some(found).flatten()
}

pub fn resolve(
    explicit: Option<Language>,
    workspace: Option<Language>,
    locale: &Locale,
) -> Language {
    explicit
        .or(workspace)
        .or_else(|| locale.language())
        .unwrap_or(Language::En)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_language_wins_over_workspace_and_locale() {
        assert_eq!(
            resolve(
                Some(Language::En),
                Some(Language::PtBr),
                &Locale {
                    lang: Some("pt_BR.UTF-8".into()),
                    ..Locale::default()
                }
            ),
            Language::En
        );
    }

    #[test]
    fn workspace_language_wins_over_locale() {
        assert_eq!(
            resolve(
                None,
                Some(Language::PtBr),
                &Locale {
                    lang: Some("en_US.UTF-8".into()),
                    ..Locale::default()
                }
            ),
            Language::PtBr
        );
    }

    #[test]
    fn locale_categories_follow_posix_precedence() {
        let locale = Locale {
            lc_all: Some("en_US.UTF-8".into()),
            lc_messages: Some("pt_BR.UTF-8".into()),
            lang: Some("pt_BR.UTF-8".into()),
        };
        assert_eq!(resolve(None, None, &locale), Language::En);
        let locale = Locale {
            lc_all: Some(String::new()),
            ..locale
        };
        assert_eq!(resolve(None, None, &locale), Language::PtBr);
    }

    #[test]
    fn unknown_workspace_values_fall_through_to_locale_then_english() {
        assert_eq!(parse_settings("# Settings\n\n- Language: fr\n"), None);
        assert_eq!(
            resolve(None, parse_settings("- Language: fr\n"), &Locale::default()),
            Language::En
        );
        assert_eq!(
            resolve(
                None,
                parse_settings("- Language: fr\n"),
                &Locale {
                    lang: Some("pt_BR.UTF-8".into()),
                    ..Locale::default()
                }
            ),
            Language::PtBr
        );
    }

    #[test]
    fn settings_parse_only_one_unfenced_supported_language_line() {
        assert_eq!(
            parse_settings("# Settings\n\n- Language: en\n"),
            Some(Language::En)
        );
        assert_eq!(parse_settings("```md\n- Language: en\n```\n"), None);
        assert_eq!(parse_settings("- Language: en\n- Language: pt-BR\n"), None);
    }
}
