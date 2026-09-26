# Mundaréu

A sandbox game for iPad, built for kids from eight years old and up: safe by
design, with no dark patterns, in Brazilian Portuguese.

Everything in this repository (code, commits, documentation) is written in
English. The game itself ships in pt-BR.

## Working on it

This project runs on the Seasoned workflow: [seasoned-skills](https://github.com/seasonedcc/seasoned-skills)
generates the standing instructions and skills Claude Code reads. After cloning,
`pnpm install` runs the sync that regenerates them.

## Repository layout

The game is a Cargo workspace with three crates:

- `crates/world` (`mundareu_world`): the pure domain. Blocks, chunks, the
  world map, terrain generation, chunk meshing, the avatar's kinematic body,
  block raycasting and the save format. It never depends on Bevy, so
  everything in it is plain Rust with plain unit tests.
- `crates/game` (`mundareu_game`): the Bevy application as a library. Its
  plugins are thin systems that wire the pure domain to rendering, input, the
  HUD and persistence. Every string the player sees lives in its `locale`
  module, in Brazilian Portuguese.
- `apps/mundareu`: the binary. It parses a few flags and calls
  `mundareu_game::run_with`. The same binary is what the iOS wrapper builds.

The rule that keeps this honest: logic goes in `crates/world`, Bevy stays in
`crates/game`.

## Running on a Mac

Install the Rust toolchain through `rustup` (the pinned version lives in
`rust-toolchain.toml`) and run:

```sh
cargo run -p mundareu
```

Move with WASD, hold the right mouse button and drag to look around, press
space to jump, left-click to break a block, right-click to place one, and
press 1 to 6 to pick a block. Touch controls are always active too.

Three flags exist so automated runs can prove the app renders:

```sh
cargo run -p mundareu -- --screenshot proof.png --exit-after-seconds 4 --log-frame-rate
```

- `--screenshot <path>`: write a PNG of the window about two seconds after
  startup.
- `--exit-after-seconds <n>`: quit cleanly after `n` seconds.
- `--log-frame-rate`: log the smoothed frame rate every two seconds.

The gates are the `full` list in `seasoned-skills.config.ts`, and they are the
same commands continuous integration runs: the formatting, clippy, tests, Biome
(`pnpm check`) and `pnpm tsc` on Ubuntu, and the iOS device compile plus the
simulator Xcode build on macOS.

## Running on an iPad

The iOS app lives in `apps/ios` as an Xcode project that
[xcodegen](https://github.com/yonaskolb/XcodeGen) generates from
`project.yml`. It has no Swift sources: a run-script phase builds the
`mundareu` binary with cargo for the chosen destination and copies it into the
app bundle as its executable.

```sh
brew install xcodegen
cd apps/ios && xcodegen generate
```

A real iPad needs your Apple development team. Copy `Signing.example.xcconfig`
to `Signing.xcconfig` (the copy is gitignored) and replace the placeholder with
your team id, found in Xcode → Settings → Accounts. Then open
`Mundareu.xcodeproj`, pick the iPad as the run destination and press Run.

The same build runs from the command line. An iPad has two identifiers and the
tools disagree on which one they want: `xcodebuild` takes the hardware
identifier that `xcrun xctrace list devices` prints, and `devicectl` takes the
CoreDevice identifier that `xcrun devicectl list devices` prints. With the iPad
connected, paired, and Developer Mode enabled on it (Settings → Privacy &
Security; it cannot be enabled from the Mac when the iPad has a passcode):

```sh
xcodebuild -project apps/ios/Mundareu.xcodeproj -scheme Mundareu \
  -configuration Release -destination 'id=<hardware identifier>' \
  -derivedDataPath apps/ios/build -allowProvisioningUpdates build
xcrun devicectl device install app --device <CoreDevice identifier> \
  apps/ios/build/Build/Products/Release-iphoneos/Mundareu.app
xcrun devicectl device process launch --device <CoreDevice identifier> \
  com.danielweinmann.mundareu
```

The first launch on a given iPad is refused until the developer profile is
trusted on the device (Settings → General → VPN & Device Management).

The simulator needs no team. Build for it from the command line with:

```sh
xcodebuild -project apps/ios/Mundareu.xcodeproj -scheme Mundareu \
  -configuration Debug -destination 'generic/platform=iOS Simulator' \
  -derivedDataPath apps/ios/build CODE_SIGNING_ALLOWED=NO build
```

and install the resulting app on a booted simulator with
`xcrun simctl install booted apps/ios/build/Build/Products/Debug-iphonesimulator/Mundareu.app`,
then launch it with
`xcrun simctl launch booted com.danielweinmann.mundareu`.

## License

The source code is licensed under the MIT license (see `LICENSE`). The game's
assets are all rights reserved (see `assets/LICENSE`). The name "Mundaréu", the
logo, and the visual identity are trademarks (see `TRADEMARKS.md`).
