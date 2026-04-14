# Mazocarta

A browser-first tactical card game written in Rust and compiled to WASM.

## Current Scope

- A full run structure across 3 sectors with branching map progression
- Deterministic combat with visible enemy intents, statuses, modules, and consumables
- Combat, elite, boss, rest, shop, and event nodes
- English and Spanish UI
- An installable web client with offline support after the first online load
- Mouse, keyboard, and touch support through Pointer Events

## Development Flow

1. Install the WASM target:

```bash
rustup target add wasm32-unknown-unknown
```

2. Run the Rust tests:

```bash
cargo test
```

3. Run the pre-publish validation pass:

```bash
make publish-check
```

4. Build the `.wasm` bundle and refresh the web host:

```bash
make build
```

5. Serve the `web/` directory:

```bash
make serve
```

6. Open `http://localhost:4173`.

## Controls

- `Click` or `tap` to select the active card, option, or map node
- `1`-`9` selects visible cards, rewards, nodes, or other numbered options depending on the screen
- `Enter` or `Space` advances the primary action on the current screen, including ending the turn in combat
- `Esc` clears combat selection, closes overlays, or returns to the title screen depending on context
- `S` opens title-screen settings
- `I` triggers title-screen install when the host supports it

## Layout

- `src/combat.rs`: combat rules and tests
- `src/app.rs`: app state, layout, input, and frame serialization
- `src/content.rs`: cards, modules, enemies, and event content
- `src/dungeon.rs`: run progression and node generation
- `web/`: static browser shell and PWA host
