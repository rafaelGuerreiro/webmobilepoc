# AGENTS.md

Guidance for agents working inside `client/`, which is a Vite + TypeScript app using Phaser for rendering and Capacitor for native packaging.

## Project map

- `src/main.ts` mounts the app and creates the top-level `App`.
- `src/app/App.ts` owns application orchestration, connection lifecycle, chat actions, and Phaser bootstrap.
- `src/app/AppView.tsx` owns the lightweight Preact DOM shell around the Phaser canvas.
- `src/game/WorldScene.ts` owns the world rendering and Phaser scene logic.
- `src/native/` contains thin native wrappers. Follow the `haptics.ts` / `keyboard.ts` pattern for new native integrations.
- `src/style.css` defines the full-screen layout, safe-area handling, and DOM overlays.
- `capacitor.config.ts` points Capacitor at `dist/`.
- `vite.config.ts` defines local dev/preview server settings.
- `src/module_bindings/` contains generated SpacetimeDB bindings; avoid hand-editing unless regeneration is intentional.

## Commands

- `npm run dev` starts the Vite dev server.
- `npm run lint` runs TypeScript checks.
- `npm run build` runs the full client validation (`tsc --noEmit && vite build`).
- From the repo root, `spacetime generate --lang typescript --out-dir client/src/module_bindings --module-path server` regenerates the typed SpacetimeDB client bindings from the Rust module in `server/`.
- `npm run cap:sync` syncs built web assets/config into native projects.
- `npm run cap:open:ios` opens the iOS project in Xcode.
- `npm run cap:open:android` opens the Android project if that platform is configured.

## What this client already gets right

- The app uses `Phaser.Scale.RESIZE`, so scene layout can react to real viewport size changes.
- `index.html` already includes `viewport-fit=cover`, which is important for full-screen native layouts on iOS.
- `src/style.css` already uses `env(safe-area-inset-*)` variables for safe-area-aware DOM chrome.
- The game canvas uses `touch-action: manipulation`, which is a good mobile default.
- Native haptics are wrapped in `src/native/haptics.ts` behind platform checks instead of being called directly from game code.
- The SpacetimeDB connection token is persisted per host/database pair, which matches the docs' recommended token reuse pattern.
- iOS keyboard behavior is handled with Capacitor Keyboard resize mode `none` plus a native wrapper in `src/native/keyboard.ts`, while the shell uses CSS safe-area variables for the Phaser mount and overlays.

## SpacetimeDB client guidance for this repo

### 1. Treat `src/module_bindings/` as generated code

The bindings under `src/module_bindings/` are generated from the server module and will be overwritten.

- Do not hand-edit files in `src/module_bindings/`.
- After changing server tables, views, reducers, procedures, or exposed types, regenerate bindings before touching client call sites.
- This repo's module lives in `server/` and is a Rust module (`server/Cargo.toml`), so use the repo-root command in the Commands section.
- Generated bindings are the source of truth for available tables, reducer names, callback signatures, and whether a table is marked as an event table.

### 2. Build the client around the local cache, not ad hoc fetches

SpacetimeDB clients maintain a local cache of subscribed rows. Reads are supposed to come from that cache after the subscription is applied.

- Wait for `subscriptionBuilder().onApplied(...)` before assuming the initial rows are ready.
- After `onApplied`, read from `conn.db.*` locally instead of treating SpacetimeDB like a request/response API.
- Keep client render state derived from the cache, as `App.ts` does with `syncFromConnection()`.
- Do not invent separate client-side truth for durable rows unless there is a clear reason.

### 3. Use `subscribeToAllTables()` for this app's always-on data

This client intentionally uses SpacetimeDB's convenience subscription for all public tables instead of maintaining a per-query list.

- For the current app shape, prefer `subscriptionBuilder().subscribeToAllTables()` over hand-maintained query arrays.
- This keeps always-on client data simple and automatically covers public tables and event tables such as `chat_bubble_v1`.
- If the client later grows enough that subscription scope becomes a performance concern, revisit this and switch to explicit query-builder subscriptions.
- If you do move back to explicit subscriptions, remember that event-style tables like `chat_bubble_v1` still need to be subscribed explicitly.

### 4. Use row observers for reactions, not for canonical storage

SpacetimeDB exposes `onInsert`, `onUpdate`, and `onDelete` callbacks on subscribed tables.

- Use row callbacks to react to cache changes, trigger UI refreshes, or maintain small derived client-side structures.
- Keep the canonical durable data in the subscription cache.
- If a feature depends on initial data, wire it through `onApplied`; row callbacks alone are not a replacement for initial synchronization.

### 5. Preserve this repo's event-table pattern

This client already uses an important SpacetimeDB pattern: `chat_bubble_v1` is generated as an event table and rows are ephemeral.

- Do not assume event-table rows will remain queryable in `.iter()`.
- For transient UX like chat bubbles, accumulate `onInsert` events into local UI state and expire them intentionally, as `App.ts` does with `liveBubbles`.
- Keep persistent world state and ephemeral event state separate.
- If you add another event-style table, decide explicitly whether it should be rendered directly from the cache or buffered client-side for UX reasons.

### 6. Reducers are the only mutation path

From the client's perspective, durable server state changes happen through reducers.

- Do not mutate subscribed rows locally and treat them as committed state.
- Add or modify server reducers when gameplay or UI actions need to change data, then regenerate bindings and call them through `conn.reducers.*`.
- Remember that reducers are transactional on the server. Design client UX around that round-trip instead of optimistic mutation unless you are intentionally layering optimistic UI on top.
- If a future feature requires external side effects, that belongs in a server procedure, not a reducer.

### 7. Respect connection and identity semantics

The TypeScript SDK runs on the browser event loop automatically; there is no manual `FrameTick()` step like there is in some other SDKs.

- Reuse tokens with `withToken(...)` to reconnect as the same identity.
- Key stored tokens per host/database pair, which this app already does.
- Use `Identity.toHexString()` for stable client-side map keys and comparisons, which this client already does through `identityKey()`.
- Remember that identity represents the user across connections, while connection IDs are per-session.
- If reliable reconnect behavior becomes important, recreate the `DbConnection` explicitly; the docs note that automatic reconnection behavior is not uniformly implemented.
- If you add a real app teardown path, disconnect the SpacetimeDB connection deliberately instead of leaving it implicit.

## Research-backed good practices for Phaser + Capacitor

### 1. Treat Phaser lifecycle and Capacitor lifecycle as separate systems

Phaser scenes own their own tweens, input plugins, display lists, cameras, and timers. Capacitor owns native app foreground/background state. If you add behavior that should pause in the background, wire it deliberately instead of assuming Phaser alone is enough.

- If you introduce audio, long-running tweens, physics, polling, or timers that should pause when the app backgrounds, bridge native lifecycle events with `@capacitor/app`.
- Use `appStateChange`, `pause`, and `resume` intentionally instead of ad hoc document visibility checks when native behavior matters.
- Remove Capacitor listeners when the owning app/module is torn down.

### 2. Keep scene ownership local and clean up aggressively

Phaser scene systems are scene-scoped. Future changes should preserve that.

- Register cleanup for `this.scale.on(...)`, `this.input.on(...)`, custom emitters, timers, and long-lived tweens.
- Use scene shutdown/destroy hooks when a scene starts accumulating listeners or long-lived objects.
- Do not store scene-owned objects in globals or DOM modules.
- If an animation can be retriggered rapidly, kill or reuse the previous tween intentionally instead of stacking them blindly.

### 3. Prefer responsive layout over hard-coded pixels

Capacitor apps run inside mobile webviews with varied aspect ratios, pixel densities, and safe areas.

- Derive layout from `this.scale.width`, `this.scale.height`, or stable scene/layout helpers.
- Re-layout UI and world elements on Phaser resize events instead of relying on CSS alone.
- Keep pixel constants local and named; use ratios/minimums for sizing where possible.
- If you add more in-canvas HUD, anchor it to the current scene size and re-position it on resize.

### 4. Respect safe areas and webview edges

This project already has the right baseline. Preserve it.

- Keep `viewport-fit=cover` in `index.html`.
- Preserve safe-area padding for DOM overlays using the existing CSS custom properties in `src/style.css`.
- Do not place important tappable controls flush against the top notch area or bottom home-indicator area.
- If a HUD element moves into Phaser canvas space, account for safe-area offsets explicitly instead of assuming a rectangular viewport.

### 5. Design primary interactions for touch first

Desktop keyboard support can be additive, but mobile input should drive the core design.

- Prefer Phaser pointer/touch events for gameplay interactions.
- Avoid hover-only or right-click-only interactions.
- Keep gestures simple and test them on real devices because WebView touch behavior can differ from desktop browsers.
- Only add multi-touch when the game actually needs it; complexity rises quickly.

### 6. Keep DOM shell responsibilities separate from Phaser responsibilities

This client already splits those concerns well.

- Keep app chrome, connection status, chat controls, and safe-area-heavy UI in Preact + CSS unless there is a strong in-canvas reason.
- Keep world rendering, animation, and object motion in Phaser.
- If DOM and Phaser must coordinate, define a clear bridge at the app layer instead of reading DOM state inside the scene render loop.
- If in-canvas UI grows significantly, consider a dedicated UI scene above the world scene rather than overloading a single scene.

### 7. Wrap native features behind small adapters

Capacitor is strongest when native integration is explicit and isolated.

- Put native plugin usage in `src/native/` wrappers, not directly in scene logic.
- Guard native-only behavior with `Capacitor.isNativePlatform()` and platform checks.
- Surface failures explicitly with targeted logging or UI feedback; do not fail silently.
- If you add plugins that launch external activities on Android, consider handling `appRestoredResult` so the app can recover cleanly if Android recreates it.

### 8. Optimize for mobile performance

Phaser inside a mobile webview is capable, but it is less forgiving than desktop Chrome.

- Prefer updating existing Phaser objects over destroy/recreate loops in hot paths.
- Avoid unnecessary allocations inside `update` or other high-frequency render paths.
- Reuse sprites, text objects, particles, and timers when possible.
- Preload assets before gameplay; avoid blocking asset work during active play unless the design explicitly supports it.
- Watch bundle size before adding heavy dependencies. Mobile startup cost matters.
- Validate on physical devices, not just desktop browsers or simulators.

### 9. Use the Capacitor workflow correctly

- `capacitor.config.ts` expects built web assets in `dist`, so build before syncing native projects.
- After changing web assets, Capacitor config, native wrappers, or plugin setup, run `npm run build && npm run cap:sync`.
- For fast on-device iteration, a dev server + Capacitor server URL can help, but do not leave machine-local dev URLs committed accidentally.
- If native iOS/Android projects are present and intentionally changed, treat them as real source files and commit those changes along with the web-side changes.

### 10. Validation expectations for this repo

- Run `npm run lint` after meaningful TypeScript changes.
- Run `npm run build` before handing work off.
- If you touch native integration or Capacitor configuration, also sync native projects and verify on-device when feasible.

## Repo-specific editing advice

- Prefer small helpers and named constants over broad class hierarchies in Phaser code.
- Keep `App.ts` focused on orchestration and state flow; put DOM markup in Preact components instead of HTML strings.
- Keep SpacetimeDB orchestration in `App.ts`: connection setup, token reuse, subscriptions, reducer calls, and cache-to-UI mapping belong there more than in Phaser scenes.
- Keep scene code deterministic and easy to redraw on resize.
- Keep persistent data flows driven by subscribed tables/views, and keep transient UX flows driven by explicit local state when the source table is event-like.
- Preserve the existing safe-area-aware CSS and touch-friendly defaults unless there is a strong reason to change them.

## Sources consulted

- SpacetimeDB clients overview: `https://spacetimedb.com/docs/clients/`
- SpacetimeDB TypeScript client reference: `https://spacetimedb.com/docs/clients/typescript/`
- SpacetimeDB code generation: `https://spacetimedb.com/docs/clients/codegen`
- SpacetimeDB connection docs: `https://spacetimedb.com/docs/clients/connection`
- SpacetimeDB subscription docs: `https://spacetimedb.com/docs/clients/subscriptions`
- SpacetimeDB SDK API overview: `https://spacetimedb.com/docs/clients/api`
- SpacetimeDB reducers docs: `https://spacetimedb.com/docs/functions/reducers`
- Capacitor Games guide: `https://capacitorjs.com/docs/guides/games`
- Capacitor App API: `https://capacitorjs.com/docs/apis/app`
- Phaser tutorial, "Bring your Phaser game to iOS and Android with Capacitor": `https://phaser.io/tutorials/bring-your-phaser-game-to-ios-and-android-with-capacitor`
- Phaser Scenes concepts: `https://docs.phaser.io/phaser/concepts/scenes`
- Ionic Capacitor Phaser starter: `https://github.com/ionic-team/capacitor-starters/tree/main/phaser`
