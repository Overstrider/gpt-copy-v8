# Project CLAUDE.md

## codedungeon

CLI pipeline available. Commands:

| Command | Use when |
|---------|----------|
| `/codedungeon --oneshot` | Small tasks: plan, code, PR, review; no task split. |
| `/codedungeon --lite` | Simple planned tasks, single-repo. Requires `.codedungeon/plans/*.md`. |
| `/codedungeon --full` | Complex features, multi-repo. Full 10-phase pipeline with architect, QA, tests, formal report. |
| `/codedungeon --rules` | Deep-read this repo, draft `.codedungeon/project-rules.md`, wait for user confirmation, then approve/compact rules. |
| `/code-review` | Standalone adversarial review on current branch. |

Without a flag, `/codedungeon` selects automatically and prints `CODEDUNGEON_MODE_SELECTED: <mode> - <reason>` before dispatch. Run `/codedungeon --rules` before first real task to discover and approve project rules. Compatibility aliases remain installed: `/one-shot`, `/side-quest`, and `/main-quest`.

Project Rules: workflows read `.codedungeon/project-rules.compact.md` when approved and include `PROJECT_RULES_STATUS`, `PROJECT_RULES_DIGEST`, and `PROJECT_RULES_READ` in handoffs.

Subagents and skills installed in `.claude/`; editable commands, phases, and mutable state live in `.codedungeon/`. CLI binary at `.claude\bin/codedungeon`.

