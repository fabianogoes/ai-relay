//! The command line: `relay-tui [--workspace <path>]`, `--version`, `--help`.
//!
//! Three arguments do not need a parsing crate.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Observe a workspace: the given path, or the current directory.
    Run { workspace: Option<PathBuf> },
    Version,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    UnknownArgument(String),
    MissingValue(&'static str),
    Repeated(&'static str),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownArgument(arg) => write!(f, "argumento desconhecido: {arg}"),
            CliError::MissingValue(option) => write!(f, "{option} precisa de um caminho"),
            CliError::Repeated(option) => write!(f, "{option} foi informado mais de uma vez"),
        }
    }
}

impl std::error::Error for CliError {}

pub const HELP: &str = "\
relay-tui: painel passivo do estado Relay de um workspace.

Uso: relay-tui [--workspace <caminho>]

Opções:
  --workspace <caminho>  workspace a observar (padrão: o diretório atual)
  -V, --version          mostra a versão
  -h, --help             mostra esta ajuda

Teclas: q, Esc ou Ctrl-C saem. O relay-tui só lê: nunca escreve em .specs/
nem em .orchestration/.
";

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
    let mut wants = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => wants = Some(Action::Help),
            "-V" | "--version" => wants = wants.or(Some(Action::Version)),
            "--workspace" => {
                let value = args.next().ok_or(CliError::MissingValue("--workspace"))?;
                set_workspace(&mut workspace, value)?;
            }
            other => match other.strip_prefix("--workspace=") {
                Some(value) => set_workspace(&mut workspace, value.to_string())?,
                None => return Err(CliError::UnknownArgument(arg)),
            },
        }
    }
    Ok(wants.unwrap_or(Action::Run { workspace }))
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
        Action::Run { workspace: workspace.map(PathBuf::from) }
    }

    #[test]
    fn no_arguments_observe_the_current_directory() {
        assert_eq!(parse_args::<_, String>([]), Ok(run(None)));
    }

    #[test]
    fn the_workspace_can_be_given_with_a_space_or_an_equals_sign() {
        assert_eq!(parse_args(["--workspace", "/work/repo"]), Ok(run(Some("/work/repo"))));
        assert_eq!(parse_args(["--workspace=/work/repo"]), Ok(run(Some("/work/repo"))));
        assert_eq!(parse_args(["--workspace", "../repo"]), Ok(run(Some("../repo"))));
        assert_eq!(parse_args(["--workspace=with space"]), Ok(run(Some("with space"))));
    }

    #[test]
    fn version_and_help_have_both_spellings() {
        assert_eq!(parse_args(["--version"]), Ok(Action::Version));
        assert_eq!(parse_args(["-V"]), Ok(Action::Version));
        assert_eq!(parse_args(["--help"]), Ok(Action::Help));
        assert_eq!(parse_args(["-h"]), Ok(Action::Help));
    }

    #[test]
    fn help_wins_over_the_rest_of_the_line() {
        assert_eq!(parse_args(["--workspace", "/x", "--help"]), Ok(Action::Help));
        assert_eq!(parse_args(["--version", "--help"]), Ok(Action::Help));
    }

    #[test]
    fn what_is_not_understood_is_an_error_not_a_guess() {
        assert_eq!(parse_args(["--exec"]), Err(CliError::UnknownArgument("--exec".into())));
        assert_eq!(parse_args(["repo"]), Err(CliError::UnknownArgument("repo".into())));
        assert_eq!(parse_args(["--workspace"]), Err(CliError::MissingValue("--workspace")));
        assert_eq!(parse_args(["--workspace="]), Err(CliError::MissingValue("--workspace")));
        assert_eq!(
            parse_args(["--workspace", "/a", "--workspace=/b"]),
            Err(CliError::Repeated("--workspace"))
        );
    }

    #[test]
    fn the_messages_are_in_portuguese() {
        assert_eq!(CliError::UnknownArgument("--x".into()).to_string(), "argumento desconhecido: --x");
        assert_eq!(CliError::MissingValue("--workspace").to_string(), "--workspace precisa de um caminho");
        assert!(HELP.contains("Uso: relay-tui"));
        assert!(version().starts_with("relay-tui "));
    }
}
