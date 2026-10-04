#!/bin/sh
# Tests for the contract the skills carry (spec 20261003-001, A-014 and A-017).
#
# Every skill that writes or validates a Relay record carries, in
# `skills/<skill>/references/`, literal copies of the sections of
# docs/PROTOCOL.md it needs. A copy that drifts from the protocol is a skill
# that writes records the protocol does not describe, so this fails when one
# does. No SKILL.md passes 39 lines either.
#
# Repository tooling, not package surface: it lives here so it is not published
# with the skills (ADR-0001). Run it with `sh .agents/tests/skill-contract.test.sh`.

ROOT=$(cd "$(dirname "$0")/../.." && pwd -P)
TMP=$(cd "$(mktemp -d)" && pwd -P)
trap 'rm -rf "$TMP"' EXIT INT TERM

# The skills that write or validate records. relay-tui-split touches none.
CONTRACT_SKILLS="relay-session relay-status relay-continue relay-spec relay-setup"
MAX_LINES=39

PASS=0
FAIL=0
ok() { PASS=$((PASS + 1)); }
bad() {
  FAIL=$((FAIL + 1))
  printf 'FAIL %s\n' "$1"
  shift
  for line in "$@"; do printf '    %s\n' "$line"; done
}

# The text of the `## <title>` section of a file: the heading and everything up
# to the next heading. A `## ` line inside a fenced block is template text, not
# a heading.
section() { # file title
  awk -v want="$2" '
    /^```/ { fence = !fence }
    !fence && /^## / { if (on) exit; on = ($0 == "## " want) }
    on { print }
  ' "$1"
}

# What may come before the first section of a reference: blank lines, the `# `
# title and `> ` notes. Anything else would be text nothing verifies.
preamble_problem() { # file
  awk '
    /^```/ { exit }
    /^## / { exit }
    /^$/ || /^# / || /^> / { next }
    { print; exit }
  ' "$1"
}

# check_reference <reference file> <protocol file> <label>
check_reference() {
  ref=$1 protocol=$2 label=$3
  sections=$(awk '/^```/ { fence = !fence } !fence && /^## / { sub(/^## /, ""); print }' "$ref")
  if [ -z "$sections" ]; then
    bad "$label: no section to compare" "$ref"
    return
  fi
  stray=$(preamble_problem "$ref")
  if [ -n "$stray" ]; then
    bad "$label: text outside a section that nothing verifies" "$stray"
  else
    ok
  fi
  printf '%s\n' "$sections" | while IFS= read -r title; do
    want=$(section "$protocol" "$title")
    have=$(section "$ref" "$title")
    if [ -z "$want" ]; then
      echo "MISSING $title"
    elif [ "$want" != "$have" ]; then
      echo "DIFFERS $title"
    else
      echo "SAME $title"
    fi
  done > "$TMP/result"
  while IFS= read -r line; do
    case $line in
      SAME*) ok ;;
      MISSING*) bad "$label: no such section in docs/PROTOCOL.md" "${line#MISSING }" ;;
      DIFFERS*) bad "$label: section differs from docs/PROTOCOL.md" "${line#DIFFERS }" ;;
    esac
  done < "$TMP/result"
}

# check_skill <skills dir> <protocol file> <skill>
check_skill() {
  dir=$1 protocol=$2 skill=$3
  files=$(ls "$dir/$skill/references/"*.md 2>/dev/null)
  if [ -z "$files" ]; then
    bad "$skill: carries no contract in references/"
    return
  fi
  for ref in $files; do
    check_reference "$ref" "$protocol" "$skill/references/$(basename "$ref")"
  done
}

check_length() { # skills dir
  for md in "$1"/*/SKILL.md; do
    lines=$(wc -l < "$md" | tr -d ' ')
    if [ "$lines" -le "$MAX_LINES" ]; then ok; else
      bad "$(basename "$(dirname "$md")")/SKILL.md has $lines lines, the limit is $MAX_LINES"
    fi
  done
}

assert_fails() { # name command...
  name=$1
  shift
  before=$FAIL
  "$@" > /dev/null
  if [ "$FAIL" -gt "$before" ]; then
    FAIL=$before
    ok
  else
    bad "self-test: $name should have failed"
  fi
}
assert_passes() { # name command...
  name=$1
  shift
  before=$FAIL
  "$@" > /dev/null
  if [ "$FAIL" -eq "$before" ]; then ok; else
    FAIL=$before
    bad "self-test: $name should have passed"
  fi
}

# --- self-tests: the check itself, on a small protocol --------------------
FX=$TMP/fx
mkdir -p "$FX/skills/a/references"
cat > "$FX/PROTOCOL.md" <<'EOF'
# Protocol

## Alpha

Text of alpha.

```markdown
# Template

## Inside a fence
- not a heading
```

More alpha.

## Beta

Text of beta.
EOF

# Same sections: passes (a `## ` inside a fence stays inside Alpha).
{ printf '# Contract\n\n> Copied verbatim.\n\n'; section "$FX/PROTOCOL.md" Alpha; section "$FX/PROTOCOL.md" Beta; } > "$FX/skills/a/references/c.md"
assert_passes "identical sections" check_skill "$FX/skills" "$FX/PROTOCOL.md" a

# One word changed.
sed 's/Text of beta/Text of BETA/' "$FX/skills/a/references/c.md" > "$FX/c2" && mv "$FX/c2" "$FX/skills/a/references/c.md"
assert_fails "a section that drifted" check_skill "$FX/skills" "$FX/PROTOCOL.md" a

# A section the protocol does not have.
{ printf '# Contract\n\n'; printf '## Gamma\n\nInvented.\n'; } > "$FX/skills/a/references/c.md"
assert_fails "a section the protocol lacks" check_skill "$FX/skills" "$FX/PROTOCOL.md" a

# Prose before the first section is text nothing verifies.
{ printf '# Contract\n\nAlways do this.\n\n'; section "$FX/PROTOCOL.md" Beta; } > "$FX/skills/a/references/c.md"
assert_fails "unverified prose" check_skill "$FX/skills" "$FX/PROTOCOL.md" a

# No references at all.
rm "$FX/skills/a/references/c.md"
assert_fails "a skill with no references" check_skill "$FX/skills" "$FX/PROTOCOL.md" a

# The length limit.
mkdir -p "$FX/len/long" "$FX/len/short"
awk 'BEGIN { for (i = 0; i < 40; i++) print "line" }' > "$FX/len/long/SKILL.md"
awk 'BEGIN { for (i = 0; i < 39; i++) print "line" }' > "$FX/len/short/SKILL.md"
assert_fails "40 lines" check_length "$FX/len/long/.."
rm -r "$FX/len/long"
assert_passes "39 lines" check_length "$FX/len"

# --- the repository --------------------------------------------------------
for skill in $CONTRACT_SKILLS; do
  check_skill "$ROOT/skills" "$ROOT/docs/PROTOCOL.md" "$skill"
done
check_length "$ROOT/skills"

printf '%s passed, %s failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
