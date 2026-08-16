# Contributing with AI Assistance

## What to optimize for

- correctness first
- minimal, reviewable diffs
- stable interfaces
- tests that prove the change
- documentation that matches the code

## Before you edit

- inspect the owning module
- trace the data flow and error flow
- confirm the target command for validation
- decide whether the change belongs in code, docs, or both

## After you edit

- run the smallest relevant test set
- run the workspace validation commands when behavior changes
- update the skill docs if boundary rules or collaboration rules changed

## Repository conventions

- `docs/skills/` held plain skill contracts (consolidated into this `ai/`
  directory alongside the collaboration guides).
- `.github/ai/AI_AGENT_GUIDE.md` is the entry point for autonomous work.
