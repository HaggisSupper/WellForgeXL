# Agent instructions

## Build location: GitHub Actions

The owner requires project builds to run on GitHub Actions with GitHub-hosted
runners, including when older instructions list local build commands.

- Do not compile, bundle, package, or run dependency installation that triggers a
  project build on the developer's computer. Do not use self-hosted runners.
- Run required security, lint, type-check, test, and build gates in the existing
  hosted workflows. Preserve required checks and platform requirements.
- Local read-only inspection and static configuration validation are allowed
  when they do not trigger a project build.
- Inspect GitHub run results and artifacts before claiming build success. If a
  hosted run is blocked, report the blocker; do not fall back to a local build.
- See `.github/BUILDING.md` for the hosted workflow entry points. A later explicit
  owner instruction can override this preference for a particular task.

