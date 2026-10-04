# Dropping work

Only `relay-spec` drops work, because dropping changes scope and scope belongs
to the spec. Nothing is ever deleted: a dropped entry stays in the records with
its reason. The rules are in `references/contract.md` ("Backlog template",
"Closing a specification", transition rule 6); this is the procedure.

## One entry

1. Read `BACKLOG.md` and the entry's spec. Only a pending entry (`[ ]` or `[!]`)
   can be dropped; an `[x]` entry is finished and a `[•]` task has a TODO, so
   stop and say so.
2. Ask the user for the reason, in one line, with the harness's native question.
   Without a reason, do not drop.
3. Look for pending entries that `needs` it. Dropping would leave them waiting
   on work that will never happen, which is a violation, so list them and ask
   whether to drop them too or to edit their `needs`. Do not drop until none is
   left.
4. Change the marker to `[-]` and append `(dropped: <reason>)` after the
   `(spec: ...)` annotation. Keep the text, the ID and the position.
5. If that was the spec's last pending entry, close the spec (below).

## A whole spec

Apply the entry steps to every pending entry of the spec with one reason for
all (or one per entry when the user prefers). A spec with no pending entry is
already closed or finished; do not reopen it. Dropping the last one closes it.

## Closing after a drop

Closing is archiving, and the closing section is the authority.

1. Every criterion of the spec must be named by a changelog record in
   `changelog/<YYYYMMDD-NNN>.md` or the legacy file, or waived. For each one
   without evidence, ask the user for a reason and prepare
   `- Waived: A-NNN - <reason>`. Only `relay-spec` writes it, and only because
   the spec has a drop.
2. Append to the spec's changelog (create the file if no record exists yet):

   ```markdown
   ## Closed <date from `date +%Y-%m-%d`>
   - <the spec's final entries, copied literally, `[x]` and `[-]`>
   - Waived: A-NNN - <reason>
   ```

3. Only then remove those entries from `BACKLOG.md`.

Report the entries dropped, the reasons, any waiver and whether the spec closed.
Write no TODO, handoff or code.
