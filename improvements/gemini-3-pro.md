# Review of `augentic/lifecycle` (alc)

This is a review of the `augentic/lifecycle` repository (specifically the `alc` CLI tool).

## Executive Summary

`alc` is a **Hub-and-Spoke orchestration tool** designed to manage software changes across multiple repositories. It centralizes "specs" (requirements/design) in one repo (the Hub) while distributing implementation tasks to target repos (the Spokes).

The design is **clean, state-driven, and pragmatic**. It avoids the complexity of a full-blown CI/CD orchestrator by keeping the "scheduler" human-driven (via CLI commands), while automating the tedious parts (git operations, context gathering, agent invocation).

---

## 1. Design Critique

### Strengths
*   **Explicit State Machine:** The `TargetState` enum in `src/status.rs` (`Pending` → `Distributed` → `Applying`...) is the strongest design element. It makes the workflow resilient. You can kill the process, restart it, and it knows exactly where it left off.
*   **Centralized Truth (The "Hub"):** keeping specs in one place (`registry.toml` + `specs/`) solves the "fragmented requirements" problem common in microservices.
*   **Idempotency:** The transition logic explicitly allows idempotent re-runs (e.g., `Distributed` → `Distributed`), which is critical for a tool that interacts with flaky external systems like Git/GitHub.

### Weaknesses
*   **The "Human Scheduler" Bottleneck:** The design relies on the user to manually drive the state machine:
    *   User runs `fan-out` → waits.
    *   User runs `apply` → waits.
    *   User runs `sync` → checks PRs.
    *   User runs `archive`.
    *   *Critique:* This is "step-by-step" rather than "declarative." You define *what* you want (`pipeline.toml`), but you still have to manually execute the *how*.
*   **Engine Abstraction (`src/engine/mod.rs`):** The `Engine` trait abstracts over `opsx`. Unless you plan to support non-OpenSpec workflows (like JIRA-based or plain-text), this is likely **YAGNI** (You Ain't Gonna Need It). It adds a layer of indirection that complicates the code without immediate benefit.

---

## 2. Implementation Critique

### Code Quality
*   **Rust Patterns:** The code uses idiomatic Rust for a CLI: `anyhow` for error handling, `clap` for parsing, and `serde` for config.
*   **Sync vs. Async:** The code appears to use `std::process` and synchronous logic. For a CLI that spends 99% of its time waiting on network (Git/GitHub/LLM), this is fine, but it prevents parallel execution of `apply` across multiple targets without using threads.

### Specific Issues
*   **Fragile Workspace Discovery:** `find_workspace_root` in `main.rs` walks up the directory tree looking for `registry.toml`. If run in a fresh terminal or wrong directory, it just `bail!`s.
    *   *Fix:* Support a global `--workspace <path>` flag or `ALC_WORKSPACE` env var.
*   **Hardcoded "Opsx" Dependency:** Despite the `Engine` trait, `main.rs` hardcodes `let engine = OpsxEngine;`. This confirms the abstraction is currently unused.

---

## 3. Missed Opportunities (Simplicity & Enhancement)

These are the "obvious" improvements to make the tool simpler to use and easier to extend.

### A. Auto-Sync (Remove the `sync` command)
**Current:** User must run `alc sync` to update local state with GitHub PR status.
**Missed Opportunity:** `alc status` and `alc apply` should **automatically** fetch the latest state from GitHub before running.
*   *Why:* Stale state is the enemy. If I merged a PR on GitHub, `alc` should know that immediately without me telling it.

### B. Parallel "Apply"
**Current:** `alc apply` processes targets in topological order (dependency order).
**Missed Opportunity:** Independent targets (siblings in the dependency graph) should be processed in **parallel**.
*   *Why:* LLM generation is slow. If Target A and Target B don't depend on each other, waiting for A to finish before starting B is a waste of time.

### C. Visualizing the Pipeline
**Current:** `alc status` prints a table.
**Missed Opportunity:** Generate a **Mermaid.js** graph of the `pipeline.toml`.
*   *Implementation:* Add `alc pipeline --graph` to output:
    ```mermaid
    graph TD
      A[Ingest Service] --> B[Processing Service]
      B --> C[Archive Service]
    ```
*   *Why:* For complex multi-repo changes, seeing the dependency graph is 10x more useful than a list.

### D. Interactive "Wizard" Mode
**Current:** The user must know the exact command sequence (`propose` -> `fan-out`...).
**Missed Opportunity:** A single `alc run <change>` command that inspects the state and runs the next logical step.
*   *Logic:*
    *   If `Pending` → run `fan-out`.
    *   If `Distributed` → run `apply`.
    *   If `Implemented` → run `sync`.
    *   If `Merged` → run `archive`.

### E. Strict Registry Validation
**Current:** `registry.toml` uses strings for `domain` and `capabilities`.
**Missed Opportunity:** Enforce a schema.
*   *Why:* Typos in `capabilities` ("ingest" vs "ingestion") will cause the agent to hallucinate or fail to find specs. Add a `[domains]` section to `registry.toml` and validate that services only use defined domains.

## Summary Recommendation

To prepare for iterative enhancement:

1.  **Flatten the Engine:** Remove the `Engine` trait unless you have a concrete second engine planned *now*.
2.  **Automate Sync:** Make `sync` an internal function called by other commands, not a user burden.
3.  **Parallelize:** Use `tokio` or `rayon` to run independent `apply` steps concurrently.
