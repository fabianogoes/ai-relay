#!/bin/sh
# Tests for skills/relay-tui-split/scripts/open-split.sh (spec 20261002-003).
#
# Repository tooling, not package surface: it lives here so the tests are not
# published with the skill (ADR-0011). Run it with `sh .agents/tests/open-split.test.sh`.
#
# Nothing real is opened. Every run is `env -i` with a PATH that holds only
# fake terminals (they log what they were called with), so the terminal the
# tests run in never leaks into a case. The whole suite runs under `sh` and,
# where it exists, `dash`: the script promises plain POSIX.

ROOT=$(cd "$(dirname "$0")/../.." && pwd -P)
SCRIPT=$ROOT/skills/relay-tui-split/scripts/open-split.sh
TMP=$(cd "$(mktemp -d)" && pwd -P)
trap 'rm -rf "$TMP"' EXIT INT TERM

BIN=$TMP/bin          # fake relay-tui, tmux, zellij, wezterm, kitten, osascript
BARE=$TMP/bare        # nothing of ours: relay-tui is "not installed"
LOG=$TMP/calls.log
mkdir -p "$BIN" "$BARE"

# A fake logs its name and arguments, one tab-separated line per call, saves
# what it was fed on stdin, and exits with $FAKE_STATUS (default 0).
for name in relay-tui tmux zellij wezterm kitten osascript; do
  cat > "$BIN/$name" <<EOF
#!/bin/sh
{ printf '%s' "$name"; for a in "\$@"; do printf '\t%s' "\$a"; done; printf '\n'; } >> "$LOG"
[ "$name" = osascript ] && cat > "$TMP/osascript.stdin"
exit \${FAKE_STATUS:-0}
EOF
  chmod +x "$BIN/$name"
done

D1=$TMP/dir\ with\ space
D2=$TMP/it\'s\ here
mkdir -p "$D1" "$D2"
HOSTPATH=/usr/bin:/bin

PASS=0
FAIL=0
SH=sh

ok() { PASS=$((PASS + 1)); }
bad() {
  FAIL=$((FAIL + 1))
  printf 'FAIL [%s] %s\n' "$SH" "$1"
  shift
  for line in "$@"; do printf '    %s\n' "$line"; done
}

assert_eq() { # name expected actual
  if [ "$2" = "$3" ]; then ok; else bad "$1" "expected: $2" "actual:   $3"; fi
}
assert_has() { # name needle haystack
  case $3 in *"$2"*) ok ;; *) bad "$1" "missing: $2" "in: $3" ;; esac
}
assert_lacks() { # name needle haystack
  case $3 in *"$2"*) bad "$1" "unexpected: $2" "in: $3" ;; *) ok ;; esac
}

# run <PATH> VAR=value... -- ARGS...   -> sets OUT and CODE; clears the call log.
run() {
  _path=$1
  shift
  : > "$LOG"
  rm -f "$TMP/osascript.stdin"
  OUT=$(env -i PATH="$_path" HOME="$TMP" "$@" 2>&1)
  CODE=$?
}

calls() { cat "$LOG"; }
tab() { printf '\t'; }
TAB=$(tab)

# `run` with the fakes on the PATH and the script under test.
with_fakes() { _vars=$1; shift; run "$BIN:$HOSTPATH" $_vars "$SH" "$SCRIPT" "$@"; }

suite() {
  # --- A-001: one dry run per terminal, in the table's form ---------------------
  with_fakes "TMUX=/tmp/tmux-1/default,1,0" --dry-run "$D1"
  assert_eq "tmux: dry-run command" "tmux split-window -h \"relay-tui --workspace '$D1'\"" "$OUT"
  assert_eq "tmux: dry-run exits 0" 0 "$CODE"
  assert_eq "tmux: dry-run executes nothing" "" "$(calls)"

  with_fakes "ZELLIJ=0" --dry-run "$D1"
  assert_eq "zellij: dry-run command" "zellij action new-pane --direction right -- relay-tui --workspace '$D1'" "$OUT"

  with_fakes "WEZTERM_PANE=7" --dry-run "$D1"
  assert_eq "wezterm: dry-run command" "wezterm cli split-pane --right --pane-id 7 -- relay-tui --workspace '$D1'" "$OUT"

  with_fakes "KITTY_WINDOW_ID=3" --dry-run "$D1"
  assert_eq "kitty: dry-run command" "kitten @ launch --location=vsplit relay-tui --workspace '$D1'" "$OUT"

  with_fakes "TERM_PROGRAM=iTerm.app ITERM_SESSION_ID=w0t1p0:ABC-123" --dry-run "$D1"
  assert_has "iterm2: dry-run is an osascript heredoc" "osascript <<'APPLESCRIPT'" "$OUT"
  assert_has "iterm2: targets the session of ITERM_SESSION_ID" 'if (unique id of s) is "ABC-123" then' "$OUT"
  assert_has "iterm2: splits vertically" "split vertically with default profile" "$OUT"
  assert_has "iterm2: writes the command in the new session" "tell n to write text \"relay-tui --workspace '$D1'\"" "$OUT"
  assert_eq "iterm2: dry-run executes nothing" "" "$(calls)"

  with_fakes "TERM_PROGRAM=WarpTerminal" --dry-run "$D1"
  assert_has "warp: dry-run is an osascript heredoc" "osascript <<'APPLESCRIPT'" "$OUT"
  assert_has "warp: checks that Warp is in front" 'does not start with "dev.warp."' "$OUT"
  assert_has "warp: Cmd+D" 'keystroke "d" using command down' "$OUT"
  assert_has "warp: types the command" "keystroke \"relay-tui --workspace '$D1'\"" "$OUT"
  assert_has "warp: Enter, after a pause so the new shell takes it" "keystroke return" "$OUT"
  assert_eq "warp: dry-run executes nothing" "" "$(calls)"

  # --- A-001: precedence, the first match wins ---------------------------------
  with_fakes "TMUX=x TERM_PROGRAM=iTerm.app ITERM_SESSION_ID=w0t0p0:Z" --dry-run "$D1"
  assert_has "tmux inside iTerm2 is tmux" "tmux split-window" "$OUT"
  with_fakes "TMUX=x ZELLIJ=0 WEZTERM_PANE=1 KITTY_WINDOW_ID=1" --dry-run "$D1"
  assert_has "tmux beats zellij, wezterm and kitty" "tmux split-window" "$OUT"
  with_fakes "ZELLIJ=0 WEZTERM_PANE=1 KITTY_WINDOW_ID=1" --dry-run "$D1"
  assert_has "zellij beats wezterm and kitty" "zellij action" "$OUT"
  with_fakes "WEZTERM_PANE=1 KITTY_WINDOW_ID=1 TERM_PROGRAM=iTerm.app" --dry-run "$D1"
  assert_has "wezterm beats kitty and iTerm2" "wezterm cli" "$OUT"
  with_fakes "KITTY_WINDOW_ID=1 TERM_PROGRAM=iTerm.app" --dry-run "$D1"
  assert_has "kitty beats iTerm2" "kitten @" "$OUT"
  with_fakes "TERM_PROGRAM=Apple_Terminal TMUX=" --dry-run "$D1"
  assert_eq "an empty TMUX is not tmux" 2 "$CODE"

  # --- the workspace is absolute and quoted ------------------------------------
  with_fakes "TMUX=x" --dry-run "$D2"
  _cmd=${OUT#tmux split-window -h }
  # The printed argument is shell syntax: evaluated, it is the command the pane
  # runs, and evaluated again it gives relay-tui the exact directory.
  : > "$LOG"
  _pane=$(env -i PATH="$BIN:$HOSTPATH" "$SH" -c "set -- $_cmd; printf %s \"\$1\"")
  _got=$(env -i PATH="$BIN:$HOSTPATH" "$SH" -c "$_pane" 2>&1; calls)
  assert_eq "an apostrophe in the path survives both shells" "relay-tui${TAB}--workspace${TAB}$D2" "$_got"

  _out=$(cd "$D1" && env -i PATH="$BIN:$HOSTPATH" TMUX=x "$SH" "$SCRIPT" --dry-run)
  assert_eq "no argument: the current directory, absolute" "tmux split-window -h \"relay-tui --workspace '$D1'\"" "$_out"
  _out=$(cd "$TMP" && env -i PATH="$BIN:$HOSTPATH" TMUX=x "$SH" "$SCRIPT" --dry-run "dir with space")
  assert_eq "a relative argument becomes absolute" "tmux split-window -h \"relay-tui --workspace '$D1'\"" "$_out"
  _out=$(env -i PATH="$BIN:$HOSTPATH" TMUX=x "$SH" "$SCRIPT" "$D1" --dry-run)
  assert_eq "--dry-run may come after the directory" "tmux split-window -h \"relay-tui --workspace '$D1'\"" "$_out"

  # --- the same actions, really executed (through the fakes) --------------------
  with_fakes "TMUX=x" "$D1"
  assert_eq "tmux: exit 0" 0 "$CODE"
  assert_eq "tmux: what ran" "tmux${TAB}split-window${TAB}-h${TAB}relay-tui --workspace '$D1'" "$(calls)"
  assert_has "tmux: says what it did" "Opened relay-tui in a tmux split" "$OUT"

  with_fakes "ZELLIJ=0" "$D1"
  assert_eq "zellij: what ran" "zellij${TAB}action${TAB}new-pane${TAB}--direction${TAB}right${TAB}--${TAB}relay-tui${TAB}--workspace${TAB}$D1" "$(calls)"

  with_fakes "WEZTERM_PANE=7" "$D1"
  assert_eq "wezterm: what ran" "wezterm${TAB}cli${TAB}split-pane${TAB}--right${TAB}--pane-id${TAB}7${TAB}--${TAB}relay-tui${TAB}--workspace${TAB}$D1" "$(calls)"

  with_fakes "KITTY_WINDOW_ID=3" "$D1"
  assert_eq "kitty: what ran" "kitten${TAB}@${TAB}launch${TAB}--location=vsplit${TAB}relay-tui${TAB}--workspace${TAB}$D1" "$(calls)"

  with_fakes "TERM_PROGRAM=iTerm.app ITERM_SESSION_ID=w0t1p0:ABC-123" "$D1"
  assert_eq "iterm2: exit 0" 0 "$CODE"
  assert_eq "iterm2: osascript was called once" "osascript" "$(calls)"
  assert_has "iterm2: the script it received" "split vertically with default profile" "$(cat "$TMP/osascript.stdin")"

  with_fakes "TERM_PROGRAM=WarpTerminal" "$D1"
  assert_eq "warp: exit 0" 0 "$CODE"
  assert_has "warp: the script it received" 'keystroke "d" using command down' "$(cat "$TMP/osascript.stdin")"

  # An AppleScript string escapes the quote of the command and a backslash.
  with_fakes "TERM_PROGRAM=iTerm.app ITERM_SESSION_ID=w0t0p0:Q" --dry-run "$D2"
  assert_has "iterm2: an apostrophe is escaped for AppleScript" "relay-tui --workspace '$TMP/it'\\\\''s here'" "$OUT"

  # --- A-002: the failures print the manual instruction and exit 2 --------------
  with_fakes "TMUX=x FAKE_STATUS=1" "$D1"
  assert_eq "tmux failure: exit 2" 2 "$CODE"
  assert_has "tmux failure: says it" "Could not open the tmux split" "$OUT"
  assert_has "tmux failure: the manual step" "Ctrl-b %" "$OUT"
  assert_has "tmux failure: the command to run" "relay-tui --workspace '$D1'" "$OUT"
  assert_eq "tmux failure: no other terminal is tried" "tmux${TAB}split-window${TAB}-h${TAB}relay-tui --workspace '$D1'" "$(calls)"

  with_fakes "KITTY_WINDOW_ID=1 FAKE_STATUS=1" "$D1"
  assert_eq "kitty failure: exit 2" 2 "$CODE"
  assert_has "kitty failure: remote control" "allow_remote_control" "$OUT"

  with_fakes "TERM_PROGRAM=iTerm.app ITERM_SESSION_ID=w0t0p0:Z FAKE_STATUS=1" "$D1"
  assert_eq "iterm2 failure: exit 2" 2 "$CODE"
  assert_has "iterm2 failure: Automation permission" "Automation" "$OUT"
  assert_eq "iterm2 failure: only osascript was tried" "osascript" "$(calls)"

  with_fakes "TERM_PROGRAM=WarpTerminal FAKE_STATUS=1" "$D1"
  assert_eq "warp failure: exit 2" 2 "$CODE"
  assert_has "warp failure: Accessibility permission" "Accessibility" "$OUT"

  with_fakes "" "$D1"
  assert_eq "unknown terminal: exit 2" 2 "$CODE"
  assert_has "unknown terminal: says it" "No supported terminal detected" "$OUT"
  assert_has "unknown terminal: the generic instruction and the command" "relay-tui --workspace '$D1'" "$OUT"
  assert_eq "unknown terminal: nothing is executed" "" "$(calls)"
  with_fakes "TERM_PROGRAM=Apple_Terminal" --dry-run "$D1"
  assert_eq "Terminal.app: exit 2 even in --dry-run" 2 "$CODE"

  # No relay-tui: nothing to open, nothing installed, the guide's link.
  for vars in "TMUX=x" "" "TERM_PROGRAM=WarpTerminal"; do
    run "$BARE:$HOSTPATH" $vars "$SH" "$SCRIPT" "$D1"
    assert_eq "no relay-tui ($vars): exit 1" 1 "$CODE"
    assert_has "no relay-tui ($vars): the guide's link" "https://github.com/fabianogoes/ai-relay/blob/main/docs/TUI.md" "$OUT"
    assert_eq "no relay-tui ($vars): nothing executed" "" "$(calls)"
  done
  run "$BARE:$HOSTPATH" TMUX=x "$SH" "$SCRIPT" --dry-run "$D1"
  assert_eq "no relay-tui: the PATH check holds in --dry-run" 1 "$CODE"
  assert_eq "no relay-tui: nothing was installed" "" "$(ls "$BARE")"

  # --- usage ---------------------------------------------------------------------
  with_fakes "TMUX=x" --bogus
  assert_eq "an unknown option exits 2" 2 "$CODE"
  with_fakes "TMUX=x" "$TMP/does not exist"
  assert_eq "a directory that does not exist exits 2" 2 "$CODE"
  assert_eq "a directory that does not exist executes nothing" "" "$(calls)"
  with_fakes "TMUX=x" "$D1" "$D2"
  assert_eq "two directories exit 2" 2 "$CODE"
}

# --- A-005: neither the script nor the skill touches a record ------------------
records() {
  _hits=$(grep -n -E '\.orchestration|\.specs|BACKLOG\.md|TODO\.md|HANDOFF\.md|CHANGELOG\.md' "$@" 2>/dev/null)
  assert_eq "no Relay record is named in the skill's files" "" "$_hits"
}
records "$SCRIPT" "$ROOT"/skills/relay-tui-split/SKILL.md

# --- the package keeps the script and not its tests ----------------------------
_tests=$(find "$ROOT/skills" -name '*test*' 2>/dev/null)
assert_eq "no test file is published inside skills/" "" "$_tests"
if [ -x "$SCRIPT" ]; then ok; else bad "the script is executable"; fi

for SH in sh dash; do
  command -v "$SH" >/dev/null 2>&1 || continue
  suite
done

printf '%s passed, %s failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
