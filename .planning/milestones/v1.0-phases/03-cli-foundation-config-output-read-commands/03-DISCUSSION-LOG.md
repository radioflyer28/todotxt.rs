# Phase 3 — Discussion Log

**Phase:** 03 — CLI Foundation — Config + Output + Read Commands
**Date:** 2026-04-15
**Areas discussed:** Config bootstrapping, List table layout, Filter + preset UX, show format + quiet scope

---

## Area 1: Config Bootstrapping

**Q:** What happens on first run when no config file exists?
**Options:** Auto-create config with defaults / Run silently without config / Error — config required
**Answer:** Auto-create config with defaults

**Q:** What's the default todo.txt path when --todo-file isn't specified and config has no todo_file key?
**Options:** ~/todo.txt default / TODOTXT_FILE env var → ~/todo.txt / Error if not configured
**Answer:** Error if not configured

**Q:** When the CLI auto-creates the config, does it include todo_file = ~/todo.txt?
**Options:** Include todo_file = ~/todo.txt in auto-created config / Leave todo_file empty
**Answer:** Include todo_file = ~/todo.txt in auto-created config

---

## Area 2: List Table Layout

**Q:** Which columns should `list` show?
**Options:** ID, priority badge, task text / ID, done, priority, due date, task text / ID and task text only
**Answer:** ID, priority badge, task text

**Q:** How should priority be rendered?
**Options:** Priority badge column + colorized task text / Colorized task text only / Priority badge column only, no text colorization
**Answer:** Priority badge column only, no text colorization

**Q:** Table style — borders or borderless?
**Options:** Borderless (aligned columns only) / Bordered table / Header row only
**Answer:** Header row only

---

## Area 3: Filter + Preset UX

**Q:** How does `list` accept inline filter terms?
**Options:** Positional args are filter tokens / Explicit --filter flag required / Mixed positional + --filter
**Answer:** (User amended) Positional args as filter tokens AND --filter flag for complex queries

**Q:** How does a user invoke a named filter preset?
**Answer:** `:name` prefix (e.g., `list :work`) — presets use colon prefix, positional args are always filter tokens

**Q:** When a positional arg like `list work` is given, how does the CLI decide if it's a preset name or a filter token?
**Options:** Preset lookup first, fall back to filter / Filter tokens always; presets use : prefix / Explicit --preset for presets
**Answer:** Filter tokens always; presets use : prefix (list :work)

**Q:** Can a preset be combined with extra filter args in the same command?
**Options:** Yes — preset + extra filter tokens / No — preset invocation is exclusive
**Answer:** Yes — preset + extra filter tokens (e.g., `list :work due:today`)

---

## Area 4: show Format + Quiet Scope

**Q:** What does `show <id>` print?
**Options:** Raw todo.txt line / Pretty field breakdown / Raw + labeled fields
**Answer:** Raw todo.txt line

**Q:** What does --quiet suppress?
**Options:** Suppress headers + info notices only / Suppress all non-data output (data only) / Suppress everything including errors
**Answer:** Suppress all non-data output (data only) — errors still surface on stderr

---
