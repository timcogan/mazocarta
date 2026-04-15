# Design Guidelines

This file is the consistency contract for Mazocarta's player-facing design.
It should describe implemented UI, UX, and screen-flow decisions so future work
extends the same game instead of drifting into ad hoc patterns.

## Purpose

- Treat this document as the source of truth for visual consistency, interaction rules, screen order, and copy tone.
- Use current implementation as the baseline. Do not document ideas from `NEXT.md` here until they are actually shipped.
- If a change alters player-facing flow or establishes a new UI convention, update this file in the same PR.

## Core Direction

- Favor clarity over feature density. Default to the smallest amount of text and UI needed to support the next decision.
- Preserve the existing visual language: black background, neon green and cyan accents, geometric panels, and restrained motion.
- Prefer strong hierarchy, spacing, and alignment over decoration.
- Reuse existing patterns before inventing new ones. New screens should feel like part of the same product, not a new theme.
- Transitional screens should stay short and focused. They are beats in the run, not summary pages.

## Interaction Rules

- Pointer and keyboard flows should stay aligned. If a primary action is clickable, it should also support an equivalent key path.
- Primary call-to-action buttons share one visual style. `Start` is the reference pattern for other primary buttons such as `Continue`, `Restart`, `Settings`, and `Install`.
- Settings and install entry points are title-screen-only. Do not add in-run versions unless that product decision changes explicitly.
- Settings currently expose only language switching between `English` and `Español`.
- Install should appear only when the host can actually act on it. Native install prompt and iOS guidance are different behaviors, but they share one button and one product concept.
- Prefer shared render and layout helpers over one-off button or panel renderers.

## Screen Flow Rules

- The main progression order after a boss should remain:
  `Boss defeated -> Boss card reward -> Boss module reward (levels 1 and 2) -> Level intro -> Map`
- The final boss keeps the card reward, then goes straight to final victory. It does not open a boss module reward.
- Boss card rewards may be skipped. Boss module rewards are mandatory.
- The level intro is a short reset beat before returning to the map. It should not become a recap or inventory screen.
- Shops allow multiple purchases in one visit and only resolve when the player chooses `Leave`.
- Event nodes resolve immediately when the player chooses an option, then return to the map without an extra confirmation screen.
- Combat consumables are part of run inventory, not cards in hand. They are used directly in combat and should not add extra confirmation friction.

## Layout And Component Rules

- Match existing typography, stroke weight, spacing, and contrast before introducing a new layout pattern.
- Reuse the shared card presentation wherever possible. Rewards, modules, and consumables should feel related even when their data differs.
- Consumables should visually reuse the card template, but remain clearly utility items: no cost number, compact width, and dedicated combat placement separate from the hand.
- Empty consumable slots should stay visually subdued, with gray text instead of the active card palette.
- Event choice cards should use smaller text than module cards and always wrap long titles and body copy within the card bounds.
- Keep `Run Info` compact and readable. Modules should render in a stable order with a little extra space before each module title.
- When adding new overlays or modals, preserve generous internal padding and clear bottom spacing. Avoid panels that feel text-cramped.
- Map symbols should stay simple and legible. Event nodes use `?`.

## Copy And Localization Rules

- Player-facing copy should stay terse, neutral, and functional. Avoid flavor text that competes with the decision the screen is asking the player to make.
- English and Spanish are both first-class UI languages. New visible text should ship in both languages at the same time.
- Keep labels, logs, and button text direct. Prefer short verbs and short noun phrases over explanatory sentences.
- Brand naming stays as `Mazocarta` in both languages.

## Maintenance Rule

- `DESIGN.md` is not a changelog. It should record durable decisions, not every implementation detail.
- If a rule in this file stops matching shipped behavior, either update the behavior or update this document immediately.
