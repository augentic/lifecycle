# Lifecycle Repo Review

## Summary

The codebase is clean, well-structured, and consistently error-handled. Module boundaries follow the workflow stages naturally, the state machine has validated transitions, and dry-run support is comprehensive. The main opportunities are around removing premature abstraction, making implicit contracts explicit, and filling gaps that would trip up real-world usage in CI and failure-recovery scenarios.

## Main Findings

### 1. Premature Engine Abstraction

The `Engine` trait has 12 methods and is threaded as `&dyn Engine` through nearly every function signature, but there is exactly one implementation: `OpsxEngine`. This adds indirection, vtable dispatch, and cognitive load for no current benefit.

`init_workspace` in `main.rs` hardcodes `openspec/changes` and `openspec/specs`, bypassing the engine abstraction entirely. If a second engine were added, `init` would silently create the wrong directories.

**Suggestion:** Replace the trait with a plain config struct (directory paths, prompt templates, artifact list). Extract a trait only when a second engine actually materializes. This would cut `Box<dyn Engine>` threading from every command signature.

### 2. Inconsistent Use of `ChangeContext`

`ChangeContext` exists to "eliminate the repeated preamble across fan-out, apply, sync, archive, and status commands," as its doc comment says. But `propose::run` doesn't use it — it manually builds `change_dir`, loads the registry separately, and gathers context on its own. This means the "one preamble" goal isn't met.

**Suggestion:** `propose` could use a lighter variant of `ChangeContext` that doesn't require an existing pipeline, or `ChangeContext` could have a `new_change()` constructor alongside `load()`.

### 3. No Session/Runtime Context

The pair `(engine, workspace)` is passed together through every function. This threading cascades into inner functions like `apply_group`, `fan_out_group`, `gather_context`, etc.

**Suggestion:** A single `Session` or `Runtime` struct holding `{ workspace, engine, concurrency, agent_backend }` would cut most signatures by 2–3 parameters and provide a natural home for shared state like a cached GitHub client.

### 4. GitHub Client Rebuilt Per API Call

`build_client()` constructs a new `Octocrab` instance on every GitHub operation. During `sync`, this means N clients for N targets. During `fan-out`, it's N clients for N groups. This is wasteful and would be a natural member of the session/runtime context.

### 5. Two Competing Dependency Systems

Targets can express dependencies via inline `depends_on` on a target or via rich `[[dependencies]]` entries with type and contract. `all_edges()` silently merges them, but the dual representation is confusing for users and error-prone (duplicate/conflicting edges).

**Suggestion:** Pick one canonical form. If rich metadata (type, contract) matters, use only `[[dependencies]]`. If ease-of-use matters, use only inline `depends_on` and add optional `type`/`contract` fields to it.

### 6. `TempDir` Doesn't Create the Directory

`TempDir::new` computes a path and cleans up any stale remnant, but never calls `create_dir_all`. The directory only comes into existence when `git clone` writes to it. This works today because `git2::RepoBuilder::clone` creates the destination, but it's a subtle contract. If any future consumer calls `TempDir::new("foo")` and then tries to write a file into `.path()`, it will fail.

**Suggestion:** Either create the dir eagerly or clearly document the deferred-creation contract.

### 7. No Retry/Resume Command

The state machine allows `Failed -> Applying` and `Failed -> Distributed`, but there's no `alc retry` command that targets failed groups. Users must manually re-run `alc apply` and reason about the filtering logic.

**Suggestion:** Add a `--retry-failed` flag to `apply`, or a dedicated `alc retry <change>` command.

### 8. Agent Backend is Env-Only, Inconsistent with CLI Flags

Every mutation command has a `--dry-run` CLI flag, but agent backend selection is only via environment variables (`ALC_AGENT_BACKEND` / `OPSX_AGENT_BACKEND`). The legacy `OPSX_AGENT_BACKEND` name alongside `ALC_AGENT_BACKEND` adds confusion.

**Suggestion:** Add an `--agent` CLI flag with a default of "claude". Keep env vars as fallbacks.

### 9. No Structured Output

All output is `println!`-based with hardcoded column formatting. No `--json` or `--output-format` flag. This limits composability for CI/CD pipelines, scripting, and dashboards — exactly the contexts where a multi-repo orchestrator is most useful.

### 10. Apply Filtering Logic is Complex and Entangled

The actionable-group filtering in `apply::run` does three things in one closure: skips already-done targets, skips pending targets, and checks upstream blocking. It also interleaves state transitions (marking targets as `Applying`) with the filtering logic.

**Suggestion:** Extract a `plan_apply()` function that returns a structured plan (groups to apply, groups to skip with reasons). This would be independently testable and easier to reason about.

### 11. Fan-Out Doesn't Handle Partial Failure Gracefully

If `fan_out` partially fails and is re-run, it skips already-distributed groups. But if a group's push succeeded while PR creation failed, the status won't have been updated (the error path catches the whole result). On retry, it will try to push again and create a duplicate PR.

**Suggestion:** Break `fan_out_group` into finer-grained steps with status updates between them, or check for existing PRs before creating.

### 12. `RepoGroup` Domain Constraint is Surprising

A mono-repo containing services from different domains (e.g., an API gateway touching both "traffic" and "train") would be rejected by the `group_by_repo` validation. This constraint isn't documented.

**Suggestion:** Either relax the constraint (domain becomes a `Vec<String>` on `RepoGroup`) or explicitly document it as a design choice in the README and `pipeline.toml` format.

### 13. Missing Test Coverage in Critical Paths

The engine module (`engine/mod.rs`, `engine/opsx.rs`) has zero tests. These are the most opinionated parts of the system — prompt construction, file distribution, artifact verification. The `distribute()` function with its recursive copy logic and the `propose_prompt()` template would benefit from snapshot tests.

There are also no integration tests for the end-to-end workflow, even with the dry-run agent backend.

### 14. Minor Implementation Issues

- **`github.rs` state formatting**: `format!("{s:?}").to_uppercase()` for PR state relies on the `Debug` representation of octocrab's state enum, which could change between versions. Pattern-match explicitly instead.
- **`propose.rs` dry-run race**: On dry-run, the change directory is created and then deleted. This leaves a window where concurrent runs could collide.
- **`git.rs` credentials**: Only supports SSH agent and `GITHUB_TOKEN`. No support for credential helpers, `gh auth token`, or other common setups. At minimum, document this limitation.
- **`all_edges` direction**: In `[[dependencies]]`, `from/to` and `depends_on` produce edges with the same direction, but the dual representation is a source of confusion.

## Highest-Impact Simplifications

| Change | Impact |
|--------|--------|
| Replace `Engine` trait with config struct | Removes `&dyn Engine` from ~15 function signatures |
| Bundle `(workspace, engine, concurrency)` into `Session` | Cuts 2–3 params from every command function |
| Pick one dependency format | Eliminates the `all_edges()` merge and dual-format confusion |
| Add `--json` output mode | Unlocks CI/scripting use cases |
| Cache GitHub client in session | Eliminates N redundant client constructions |
| Add `--retry-failed` to apply | Surfaces the `Failed -> Applying` transition as a command |
| Add engine tests with fixtures | Catches prompt/distribution regressions |
