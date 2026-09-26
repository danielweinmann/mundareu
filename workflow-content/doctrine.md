Mundaréu is a sandbox game for iPad, for kids from eight years old and up. It
exists because the family behind it wanted the fun of the big block-building
sandboxes without their dark patterns and safety problems. Nothing in the game
may prey on a child: no ads, no manipulative monetization, no chat with
strangers, no third-party analytics or tracking, no mechanic that nudges a kid
to keep playing or to come back. Judge every feature against that before anything
else. The game ships in Brazilian Portuguese only; everything in the repository
(code, commits, documentation, pull requests) is written in English.

## Engine and stack

The game is Rust on Bevy, ruled after a spike measured on a real iPad. The
ruling behind that choice governs every future dependency as well: everything
the game is built on is open source, and nothing behind the game is a service
someone else runs. No proprietary engine or SDK, no analytics or
crash-reporting service. The README's repository layout states the
architecture rule (game logic in `crates/world`, Bevy only in `crates/game`)
and it holds for every change.

## Verify Bevy against its sources

Bevy changes its API every release, and what an API does often differs from
what its name suggests. Before using any Bevy API, read it in the pinned
version's source under `~/.cargo/registry/src/*/bevy_*-<version>/` (or docs.rs
for that exact version), never from memory or from another version's
migration guide.
