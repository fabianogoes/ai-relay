#!/bin/sh
# Opens relay-tui in a split pane to the right of the current terminal.
#
#   open-split.sh [--dry-run] [DIR]
#
# DIR is the workspace to observe (default: the current directory). The terminal
# is detected from environment variables; the first match wins, and multiplexers
# come before emulators because they run inside them. Reads and writes no Relay
# record, and installs nothing.
#
# Exit status: 0 opened the pane (or --dry-run), 1 relay-tui is not on PATH,
# 2 unrecognized terminal, split failed, or bad usage (instructions printed).

GUIDE="https://github.com/fabianogoes/ai-relay/blob/main/docs/TUI.md"
DRY=""
DIR_ARG=""

usage() {
  printf 'usage: open-split.sh [--dry-run] [DIR]\n' >&2
}

for arg in "$@"; do
  case $arg in
    --dry-run) DRY=1 ;;
    -h | --help) usage; exit 0 ;;
    -*) printf 'open-split.sh: unknown option: %s\n' "$arg" >&2; usage; exit 2 ;;
    *)
      if [ -n "$DIR_ARG" ]; then
        printf 'open-split.sh: only one directory is accepted\n' >&2
        usage
        exit 2
      fi
      DIR_ARG=$arg
      ;;
  esac
done

# --- quoting -----------------------------------------------------------------

# A string as one single-quoted shell word.
sq() {
  printf "'%s'" "$(printf %s "$1" | sed "s/'/'\\\\''/g")"
}

# A string as one double-quoted shell word.
dq() {
  printf '"%s"' "$(printf %s "$1" | sed 's/[\\"$`]/\\&/g')"
}

# A string escaped for an AppleScript string literal.
as() {
  printf %s "$1" | sed 's/[\\"]/\\&/g'
}

# --- relay-tui and the workspace --------------------------------------------

if ! command -v relay-tui >/dev/null 2>&1; then
  printf 'relay-tui is not on your PATH, so there is nothing to open.\n'
  printf 'Download it (one binary, nothing else to install): %s\n' "$GUIDE"
  exit 1
fi

DIR_ARG=${DIR_ARG:-.}
case $DIR_ARG in -*) DIR_ARG=./$DIR_ARG ;; esac
if ! DIR=$(cd "$DIR_ARG" 2>/dev/null && pwd); then
  printf 'open-split.sh: not a directory: %s\n' "$DIR_ARG" >&2
  exit 2
fi

# The command the new pane runs. The path is always quoted: it may have spaces,
# and `--workspace` means the pane does not depend on where it starts.
CMD="relay-tui --workspace $(sq "$DIR")"

# --- the terminals -----------------------------------------------------------
# Each `open_*` prints the command it would run under --dry-run and runs it
# otherwise; its status is the status of the split.

open_tmux() {
  if [ -n "$DRY" ]; then
    printf 'tmux split-window -h %s\n' "$(dq "$CMD")"
  else
    tmux split-window -h "$CMD"
  fi
}

open_zellij() {
  if [ -n "$DRY" ]; then
    printf 'zellij action new-pane --direction right -- relay-tui --workspace %s\n' "$(sq "$DIR")"
  else
    zellij action new-pane --direction right -- relay-tui --workspace "$DIR"
  fi
}

open_wezterm() {
  if [ -n "$DRY" ]; then
    printf 'wezterm cli split-pane --right --pane-id %s -- relay-tui --workspace %s\n' \
      "$WEZTERM_PANE" "$(sq "$DIR")"
  else
    wezterm cli split-pane --right --pane-id "$WEZTERM_PANE" -- relay-tui --workspace "$DIR"
  fi
}

open_kitty() {
  if [ -n "$DRY" ]; then
    printf 'kitten @ launch --location=vsplit relay-tui --workspace %s\n' "$(sq "$DIR")"
  else
    kitten @ launch --location=vsplit relay-tui --workspace "$DIR"
  fi
}

# In the session that ran the script (ITERM_SESSION_ID is `w0t0p0:<id>`), split
# vertically and type the command into the new session; `write text` runs it in
# the new pane's login shell, with the user's PATH.
iterm_script() {
  session_id=${ITERM_SESSION_ID#*:}
  printf 'tell application id "com.googlecode.iterm2"\n'
  if [ -n "$ITERM_SESSION_ID" ]; then
    printf '  repeat with w in windows\n'
    printf '    repeat with t in tabs of w\n'
    printf '      repeat with s in sessions of t\n'
    printf '        if (unique id of s) is "%s" then\n' "$(as "$session_id")"
    printf '          tell s to set n to (split vertically with default profile)\n'
    printf '          tell n to write text "%s"\n' "$(as "$CMD")"
    printf '          return\n'
    printf '        end if\n'
    printf '      end repeat\n'
    printf '    end repeat\n'
    printf '  end repeat\n'
    printf '  error "the iTerm2 session %s was not found"\n' "$(as "$session_id")"
  else
    printf '  tell current session of current window\n'
    printf '    set n to (split vertically with default profile)\n'
    printf '  end tell\n'
    printf '  tell n to write text "%s"\n' "$(as "$CMD")"
  fi
  printf 'end tell\n'
}

# Warp has no split command. Cmd+D is its split-right shortcut, sent by System
# Events to whatever app is in front, so first check that it is Warp.
warp_script() {
  printf 'tell application "System Events"\n'
  printf '  set frontApp to first application process whose frontmost is true\n'
  printf '  if (bundle identifier of frontApp) does not start with "dev.warp." then\n'
  printf '    error "Warp is not the frontmost application"\n'
  printf '  end if\n'
  printf '  keystroke "d" using command down\n'
  printf '  delay 0.6\n'
  printf '  keystroke "%s"\n' "$(as "$CMD")"
  printf '  delay 0.4\n'
  printf '  key code 36\n'
  printf 'end tell\n'
}

open_applescript() {
  if [ -n "$DRY" ]; then
    printf 'osascript <<'\''APPLESCRIPT'\''\n'
    "$1"
    printf 'APPLESCRIPT\n'
  else
    "$1" | osascript
  fi
}

# --- what to do when it does not work ---------------------------------------

manual() {
  printf '%s\n' "$1"
  printf 'Then run:\n  %s\n' "$CMD"
}

manual_for() {
  case $1 in
    tmux) manual 'Split the window (Ctrl-b %), or open a pane to the right of this one.' ;;
    zellij) manual 'Open a pane to the right of this one (Alt-n, or Ctrl-p then n).' ;;
    wezterm) manual 'Split this pane to the right (WezTerm: Ctrl+Shift+Alt+%).' ;;
    kitty) manual 'Turn on remote control (allow_remote_control yes in kitty.conf) and try again, or open a split (Ctrl+Shift+Enter).' ;;
    iterm2) manual 'Split the pane vertically (Cmd+D). If the split failed with a permission error, allow your terminal under System Settings > Privacy & Security > Automation.' ;;
    warp) manual 'Split the pane to the right (Cmd+D). If the split failed, bring Warp to the front and allow your terminal under System Settings > Privacy & Security > Accessibility.' ;;
    *) manual 'Open a split pane to the right of the harness.' ;;
  esac
}

# --- detection: the first match wins ----------------------------------------

if [ -n "${TMUX:-}" ]; then
  terminal=tmux; open=open_tmux
elif [ -n "${ZELLIJ:-}" ]; then
  terminal=zellij; open=open_zellij
elif [ -n "${WEZTERM_PANE:-}" ]; then
  terminal=wezterm; open=open_wezterm
elif [ -n "${KITTY_WINDOW_ID:-}" ]; then
  terminal=kitty; open=open_kitty
elif [ "${TERM_PROGRAM:-}" = "iTerm.app" ]; then
  terminal=iterm2; open='open_applescript iterm_script'
elif [ "${TERM_PROGRAM:-}" = "WarpTerminal" ]; then
  terminal=warp; open='open_applescript warp_script'
else
  printf 'No supported terminal detected (tmux, zellij, WezTerm, kitty, iTerm2, Warp).\n'
  manual_for generic
  exit 2
fi

if [ -n "$DRY" ]; then
  $open
  exit 0
fi

if $open; then
  printf 'Opened relay-tui in a %s split, watching %s\n' "$terminal" "$DIR"
  exit 0
fi

printf 'Could not open the %s split.\n' "$terminal"
manual_for "$terminal"
exit 2
