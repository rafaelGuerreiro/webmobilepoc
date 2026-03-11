# AGENTS.md - webmobiledb-core coding guidelines

## Purpose

Use this file as a mistake-prevention checklist for backend changes in `sdks/webmobiledb-core/**`.
Prefer rules that stay valid over time, not feature-by-feature notes.

## Scope

- Applies to all code under `sdks/webmobiledb-core/**`.
- If a task touches other crates, keep these rules for core logic unless explicitly overridden by the user.

## Core engineering rules

1. **No surprise dependency/tooling changes**
   - Do not add dependencies or edit `Cargo.toml` unless explicitly requested.
   - Use existing task workflow (`task check`, `task test`).

2. **No panic-driven logic**
   - Avoid `panic!`, `unwrap()`, and `expect()` in service/reducer paths.
   - Return `ServiceResult<T>` and map errors intentionally.

3. **Reducers are thin and live in `reducers.rs`**
   - Domain reducers belong in `{domain}/reducers.rs`.
   - Reducers should validate input, enforce access, extract caller context, and delegate.
   - Keep decision-heavy logic in services.

4. **Services own state transitions**
   - Business logic and table mutations belong in `{domain}/services.rs`.
   - Pass user/session identity into service methods explicitly.
   - Do not hide reducer context assumptions in call sites.

5. **Respect domain boundaries**
   - A domain writes its own tables.
   - Cross-domain writes go through the owning domain’s service API, not direct table updates.

6. **Use domain-specific typed errors**
   - In services/validation flows, do not manually build `ServiceError` with formatted strings.
   - Define per-domain `thiserror` enums (`XxxError`) inside that domain’s `services.rs`.
   - Expose helper constructors that map to `ServiceError` via `ErrorMapper`.

7. **Validation should be reusable and explicit**
   - Keep shared guards in `extend/validate.rs` (for example `require_*` checks).
   - Use these guards at reducer boundaries before service delegation.
   - Prefer one normalization/validation function per input contract (avoid duplicated trim/parse paths).

8. **Model sensitive/progression data separately**
   - Keep identity/profile state separate from private progression/combat/session state.
   - Use dedicated private tables for private/stat data, and wire joins in services/views.

9. **Session state is explicit**
   - If “current selection” exists (character/session/etc.), store it in a dedicated table.
   - Do not infer session state indirectly from broad flags.

10. **Canonical vs display values**
    - For user-facing names, keep display value and canonical unique handle distinct.
    - Canonical value should be deterministic and backed by a unique index.

11. **Views are curated read surfaces**
    - Keep public views in `{domain}/views.rs`.
    - Return only what clients should read; keep tables private by default.
    - Scope caller-specific views by `ctx.sender()` when identity matters.

12. **Constants belong in `src/constants.rs`**
    - Validation bounds and default gameplay values should be centralized.
    - Avoid scattering magic numbers across reducers/services.

13. **Time/randomness discipline**
    - Use `ctx.timestamp` as authoritative time.
    - Use existing context/random helpers; avoid system time and external RNGs.

14. **Lint suppression requires approval**
    - Ask the user before adding any `#[allow(...)]` for compiler or clippy warnings.

## Change checklist (before finalize)

- Are reducers still thin and in `reducers.rs`?
- Are new errors represented by typed domain enums instead of manual `ServiceError` formatting?
- Did a schema/model split require corresponding service and view updates?
- Are constants centralized (no new magic numbers)?
- Did you run `task test` from the repository root? Prefer `task test` over crate-scoped
  `cargo test -p …` so the whole workspace is validated together.
