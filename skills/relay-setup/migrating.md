# Migrating the legacy changelog

An existing `.orchestration/CHANGELOG.md` stays valid to read and never receives
a new record. Moving its records into one file per spec is the only movement of
a record the protocol allows, and only `relay-setup` does it, after the user
confirms. The rules are in `references/contract.md` ("Changelog template",
"Legacy changelog"); this is the procedure.

## 1. Plan, without writing

A record is a header line `## <date> - T-NNN - <title>` and every line up to the
next such header or the end of the file, without the blank lines that end it. A
record belongs to the spec of its `- Spec: <path>` line, and its id is the first
`YYYYMMDD-NNN` in the path. It can move only when that spec exists as a file in
`.specs/` whose name starts with the id. A record with no `Spec`, with an older
name that has no id, or with a spec that is not in `.specs/` stays in the legacy
file.

Count the records per destination `changelog/<id>.md` and the ones that stay.
Ask one native question: migrate N records into M files (listing the files and
how many records each gets, and the records that stay with their headers), or
leave the legacy file as it is. Without a yes, write nothing.

## 2. Write, in this order

1. For each destination, create `.orchestration/changelog/<id>.md` when it does
   not exist, with the line `# Change log <id>` and a blank line.
2. Append each record to its destination in the original order, **text
   unchanged**, one blank line between records. If the destination already holds
   that exact record, do not repeat it (an interrupted run resumes here).
3. Only after every record is verified in its destination, remove the moved
   records from `CHANGELOG.md`. A crash before this step leaves a record in two
   places, never in none, and the next run completes it.

Leave the `# Change log` heading and the records that stay. Do not delete the
file, even when nothing stays.

## 3. Verify and report

Re-read both sides. The moved records must appear byte for byte in their files,
and the count of records before the run must equal the count moved plus the
count that stayed. Report the files created, the records moved and the ones left
in the legacy file with the reason. A second run finds no movable record and
changes nothing: say so and write nothing.
