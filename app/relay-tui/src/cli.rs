//! The command line: `relay-tui [--workspace <path>]`, `--version`, `--help`.
//!
//! Three arguments do not need a parsing crate.

use crate::language::Language;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Observe a workspace: the given path, or the current directory.
    Run {
        workspace: Option<PathBuf>,
        language: Option<Language>,
    },
    Version,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    UnknownArgument(String),
    MissingValue(&'static str),
    Repeated(&'static str),
    InvalidLanguage(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownArgument(arg) => write!(f, "argumento desconhecido: {arg}"),
            CliError::MissingValue(option) => write!(f, "{option} precisa de um caminho"),
            CliError::Repeated(option) => write!(f, "{option} foi informado mais de uma vez"),
            CliError::InvalidLanguage(value) => {
                write!(f, "idioma inválido: {value} (use en ou pt-BR)")
            }
        }
    }
}

impl CliError {
    pub fn message(&self, language: Language) -> String {
        match language {
            Language::PtBr => self.to_string(),
            Language::En => match self {
                CliError::UnknownArgument(arg) => format!("unknown argument: {arg}"),
                CliError::MissingValue(option) if *option == "--lang" => {
                    "--lang needs a value (en or pt-BR)".into()
                }
                CliError::MissingValue(option) => format!("{option} needs a path"),
                CliError::Repeated(option) => format!("{option} was supplied more than once"),
                CliError::InvalidLanguage(value) => {
                    format!("invalid language: {value} (use en or pt-BR)")
                }
            },
        }
    }
}

impl std::error::Error for CliError {}

pub const HELP: &str = "\
relay-tui: painel passivo do estado Relay de um workspace.

Uso: relay-tui [--workspace <caminho>] [--lang <en|pt-BR>]

Opções:
  --workspace <caminho>  workspace a observar (padrão: o diretório atual)
  --lang <en|pt-BR>       idioma da interface (padrão: workspace, depois locale)
  -V, --version          mostra a versão
  -h, --help             mostra esta ajuda

Teclas: q, Esc ou Ctrl-C saem. O relay-tui só lê: nunca escreve em .specs/
nem em .orchestration/.
";

pub const HELP_EN: &str = "\
relay-tui: passive view of a workspace's Relay state.

Usage: relay-tui [--workspace <path>] [--lang <en|pt-BR>]

Options:
  --workspace <path>     workspace to observe (default: current directory)
  --lang <en|pt-BR>       interface language (default: workspace setting, then locale)
  -V, --version           show version
  -h, --help              show this help

Keys: q, Esc or Ctrl-C to quit. relay-tui only reads: it never writes to .specs/
or .orchestration/.
";

pub fn help(language: Language) -> &'static str {
    match language {
        Language::En => HELP_EN,
        Language::PtBr => HELP,
    }
}

pub fn version() -> String {
    format!("relay-tui {}", env!("CARGO_PKG_VERSION"))
}

/// Parses the arguments after the program name. `--help` and `--version` win
/// over everything else on the line, as is usual.
pub fn parse_args<I, S>(args: I) -> Result<Action, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let mut workspace: Option<PathBuf> = None;
    let mut language: Option<Language> = None;
    let mut wants = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => wants = Some(Action::Help),
            "-V" | "--version" => wants = wants.or(Some(Action::Version)),
            "--workspace" => {
                let value = args.next().ok_or(CliError::MissingValue("--workspace"))?;
                set_workspace(&mut workspace, value)?;
            }
            "--lang" => {
                let value = args.next().ok_or(CliError::MissingValue("--lang"))?;
                set_language(&mut language, value)?;
            }
            other => match other.strip_prefix("--workspace=") {
                Some(value) => set_workspace(&mut workspace, value.to_string())?,
                None => match other.strip_prefix("--lang=") {
                    Some(value) => set_language(&mut language, value.to_string())?,
                    None => return Err(CliError::UnknownArgument(arg)),
                },
            },
        }
    }
    Ok(wants.unwrap_or(Action::Run {
        workspace,
        language,
    }))
}

fn set_language(slot: &mut Option<Language>, value: String) -> Result<(), CliError> {
    let language = Language::parse(&value).ok_or(CliError::InvalidLanguage(value))?;
    if slot.replace(language).is_some() {
        return Err(CliError::Repeated("--lang"));
    }
    Ok(())
}

fn set_workspace(slot: &mut Option<PathBuf>, value: String) -> Result<(), CliError> {
    if value.is_empty() {
        return Err(CliError::MissingValue("--workspace"));
    }
    if slot.replace(PathBuf::from(value)).is_some() {
        return Err(CliError::Repeated("--workspace"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(workspace: Option<&str>) -> Action {
        Action::Run {
            workspace: workspace.map(PathBuf::from),
            language: None,
        }
    }

    #[test]
    fn no_arguments_observe_the_current_directory() {
        assert_eq!(parse_args::<_, String>([]), Ok(run(None)));
    }

    #[test]
    fn the_workspace_can_be_given_with_a_space_or_an_equals_sign() {
        assert_eq!(
            parse_args(["--workspace", "/work/repo"]),
            Ok(run(Some("/work/repo")))
        );
        assert_eq!(
            parse_args(["--workspace=/work/repo"]),
            Ok(run(Some("/work/repo")))
        );
        assert_eq!(
            parse_args(["--workspace", "../repo"]),
            Ok(run(Some("../repo")))
        );
        assert_eq!(
            parse_args(["--workspace=with space"]),
            Ok(run(Some("with space")))
        );
    }

    #[test]
    fn version_and_help_have_both_spellings() {
        assert_eq!(parse_args(["--version"]), Ok(Action::Version));
        assert_eq!(parse_args(["-V"]), Ok(Action::Version));
        assert_eq!(parse_args(["--help"]), Ok(Action::Help));
        assert_eq!(parse_args(["-h"]), Ok(Action::Help));
    }

    #[test]
    fn language_flag_is_recognized() {
        assert_eq!(
            parse_args(["--lang", "en"]),
            Ok(Action::Run {
                workspace: None,
                language: Some(Language::En)
            })
        );
        assert_eq!(
            parse_args(["--lang=pt-BR"]),
            Ok(Action::Run {
                workspace: None,
                language: Some(Language::PtBr)
            })
        );
        assert_eq!(
            parse_args(["--lang=fr"]),
            Err(CliError::InvalidLanguage("fr".into()))
        );
        assert_eq!(
            parse_args(["--lang"]),
            Err(CliError::MissingValue("--lang"))
        );
        assert_eq!(
            parse_args(["--lang=en", "--lang=pt-BR"]),
            Err(CliError::Repeated("--lang"))
        );
    }

    #[test]
    fn help_wins_over_the_rest_of_the_line() {
        assert_eq!(
            parse_args(["--workspace", "/x", "--help"]),
            Ok(Action::Help)
        );
        assert_eq!(parse_args(["--version", "--help"]), Ok(Action::Help));
    }

    #[test]
    fn what_is_not_understood_is_an_error_not_a_guess() {
        assert_eq!(
            parse_args(["--exec"]),
            Err(CliError::UnknownArgument("--exec".into()))
        );
        assert_eq!(
            parse_args(["repo"]),
            Err(CliError::UnknownArgument("repo".into()))
        );
        assert_eq!(
            parse_args(["--workspace"]),
            Err(CliError::MissingValue("--workspace"))
        );
        assert_eq!(
            parse_args(["--workspace="]),
            Err(CliError::MissingValue("--workspace"))
        );
        assert_eq!(
            parse_args(["--workspace", "/a", "--workspace=/b"]),
            Err(CliError::Repeated("--workspace"))
        );
    }

    #[test]
    fn the_messages_are_in_portuguese() {
        assert_eq!(
            CliError::UnknownArgument("--x".into()).to_string(),
            "argumento desconhecido: --x"
        );
        assert_eq!(
            CliError::MissingValue("--workspace").to_string(),
            "--workspace precisa de um caminho"
        );
        assert!(HELP.contains("Uso: relay-tui"));
        assert!(version().starts_with("relay-tui "));
    }

    #[test]
    fn help_and_errors_follow_the_selected_language() {
        assert!(help(Language::En).contains("Usage:"));
        assert!(help(Language::PtBr).contains("Uso:"));
        assert_eq!(
            CliError::UnknownArgument("--bad".into()).message(Language::En),
            "unknown argument: --bad"
        );
        assert_eq!(
            CliError::InvalidLanguage("fr".into()).message(Language::En),
            "invalid language: fr (use en or pt-BR)"
        );
    }
}
