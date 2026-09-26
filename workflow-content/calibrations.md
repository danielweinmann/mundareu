# Subagent calibrations

Calibrations are stated relative to the Definition of Done and accrete
through pull requests as sessions learn what this project needs. Each figure
is the subagent's total context at handback, measured from its transcript.

- A builder that creates a Cargo crate or two from a detailed charter (six
  commits: workspace, pure domain crate with tests, Bevy game crate, binary,
  screenshot proof, README) lands around 235k tokens with the fast cargo
  checks run in its own foreground. Half of that is reading Bevy sources to
  verify APIs; charters that name the exact files to read keep it there.
- A builder that adds the iOS wrapper, CI workflow and gate configuration
  (five commits plus a rebase) lands around 170k tokens.
- A review-fix builder working through eleven ruled findings across both
  crates, each red-then-green with its own commit, lands around 150k tokens.
- A read-only review finder over a ten-thousand-line diff (one angle: domain,
  game, tooling, or quality) lands between 75k and 130k tokens; four such
  finders in parallel cover the diff.
