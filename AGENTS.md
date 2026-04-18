# Repository Guidelines

## Project Structure & Module Organization

- `src/` contains the Rust game code. Keep combat rules in `src/combat.rs`, encounter content in `src/content.rs`, dungeon flow in `src/dungeon.rs`, and browser-facing app state and input handling in `src/app.rs`.
- `src/lib.rs` exposes the WASM entry points used by the browser host.
- `web/` holds the static shell: `index.html`, `index.js`, `styles.css`, artwork, and the generated `mazocarta.wasm`.
- `scripts/build-web.sh` builds the release WASM bundle and copies it into `web/`.

## Build, Test, and Development Commands

- `rustup target add wasm32-unknown-unknown` installs the required target for browser builds.
- `cargo test` runs the Rust unit tests across combat, content, and dungeon modules.
- `make build` compiles the release WASM and refreshes `web/mazocarta.wasm`.
- `make serve` serves `web/` at `http://localhost:4173`.
- `make debug` builds, enables `web/.debug-mode.json`, and starts the local server with debug mode on.

## Coding Style & Naming Conventions

- Run `cargo fmt` before submitting. Follow standard Rust formatting: 4-space indentation, trailing commas where rustfmt would add them, `snake_case` for functions and modules, and `PascalCase` for types and enums.
- Match the existing module style: compact data types, explicit enums for state transitions, and small helper methods close to the logic they support.
- Do not hand-edit generated build output unless you are intentionally updating the shipped browser artifact; prefer changing `src/` or `web/` source files and rebuilding.

## Migration And Compatibility Policy

- Apply these rules to any save, load, restore, or format-compatibility change.
- Keep legacy compatibility logic at the save/load boundary, primarily in `src/save.rs` and save/restore helpers in `src/app.rs`. Do not spread migration branches through gameplay, render, or input code.
- The current save policy supports only the exact current `save_format_version`. Within a format version, additive compatibility shims such as `#[serde(default)]` or restore fallbacks are acceptable only when the saved data remains semantically compatible.
- Renames, structural changes, or semantic changes to saved data must bump `SAVE_FORMAT_VERSION`.
- Every migration or compatibility shim must document three things in a nearby comment and in the PR description: what legacy format or field it supports, what fallback behavior is expected, and the exact condition for removing it.
- Use a concrete removal trigger for temporary compatibility code, for example: `MIGRATION(save vN): ... Remove when minimum supported save format > N.`
- If a PR raises the minimum supported save format, remove the obsolete migration code and its legacy tests or fixtures in the same PR.
- If the project later adds explicit cross-version save migration, keep the default compatibility window short: support the current format and at most one immediately previous format unless a documented product requirement says otherwise.

## Testing Guidelines

- Keep tests near the code they validate inside `#[cfg(test)] mod tests` blocks. Existing examples live in `src/combat.rs`, `src/content.rs`, and `src/dungeon.rs`.
- Name tests descriptively in `snake_case`, focusing on behavior, for example `enemy_intent_advances_after_turn`.
- Run `cargo test` before opening a PR. Add or update tests for rule changes, balance logic, or map generation behavior.

## Commit & Pull Request Guidelines

- Mirror the existing history: short, imperative commit subjects such as `Improve map generation` or `Add reward after each battle`.
- Keep commits focused on one gameplay, UI, or build concern.
- PRs should explain the user-visible change, list validation performed, and include screenshots or a short recording when `web/` visuals or interactions change.
