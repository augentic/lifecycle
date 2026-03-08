# Critique of `augentic/lifecycle` (`alc`)

**Agent:** gemini-3-pro-preview
**Date:** 2026-03-08

## Executive Summary

`alc` is a well-structured, ambitious tool for multi-repo orchestration. It effectively implements a "hub-and-spoke" architecture where a central repository manages the planning phase, and changes are distributed to target repositories for execution. The codebase is clean, uses modern Rust idioms (2024 edition), and leverages the type system well to enforce state transitions.

However, the tool currently feels like a "single-user prototype" rather than a general-purpose platform. Its reliance on the `claude` CLI and hardcoded prompts severely limits its portability and iterative potential.

---

## 1. Design & Architecture

### Strengths
*   **Hub-and-Spoke Model:** The separation between the "hub" (planning, registry) and "spokes" (target repos) is excellent. It allows for a holistic view of a system change before a single line of code is written.
*   **File-Based State:** Using `status.toml` and directory structures (`openspec/changes/...`) for state management is transparent and git-ops friendly. It allows users to debug the state of a migration simply by looking at the file system.
*   **Engine Abstraction:** The `Engine` trait (`src/engine/mod.rs`) is the right abstraction. It theoretically allows swapping out the "Opsx" implementation for other strategies, though currently only one exists.

### Weaknesses
*   **Agent Invocation via CLI:** `src/agent.rs` shells out to the `claude` binary. This is a significant design fragility.
    *   **Portability:** It requires every user to have the `claude` CLI installed and authenticated.
    *   **Control:** You lose fine-grained control over the API interaction (e.g., temperature, stop sequences, usage metrics).
    *   **Observability:** You capture `stderr` but do not stream the agent's "thinking" process to the user, leading to a "frozen screen" experience during long generations.
*   **Hardcoded Prompts:** The prompts are embedded in Rust `format!` macros inside `src/engine/opsx.rs`. This is the single biggest barrier to "iterative enhancement." To improve the prompt, you must recompile the binary.

## 2. Implementation Critique

### Strengths
*   **Async/Sync Bridging:** The use of `tokio::task::spawn_blocking` for `git2` operations in `src/git.rs` is correct and prevents blocking the async runtime.
*   **Error Context:** The pervasive use of `anyhow::Context` ensures that errors are traceable (e.g., "failed to read specs from clone" -> "cloning https://...").

### Areas for Improvement
*   **Git Safety:** In `src/git.rs`, `add_commit_push_sync` uses `index.add_all(["*"], ...)`:
    ```rust
    index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)
    ```
    While convenient, this is dangerous in an automated tool. It risks committing untracked files (like `.DS_Store` or local config) if `.gitignore` is not perfectly set up. It should likely be more selective about what it commits (e.g., only the `specs/` directory or specific modified files).
*   **Context Scalability:** `src/propose.rs` reads *all* markdown files in the `specs` directory of every target repo:
    ```rust
    // src/propose.rs
    output.push_str(&format!("\n## {}\n{}\n", rel.display(), content));
    ```
    As repositories grow, this context window will explode. There is no mechanism to select *relevant* specs or summarize them.

## 3. Missed Opportunities

These are the "obvious" changes that would make the tool significantly simpler to use and evolve.

### A. Externalized Prompt Templates (The "Iterative" Fix)
**Current:** Hardcoded strings in `src/engine/opsx.rs`.
**Opportunity:** Use a templating engine like `minijinja` or `tera`. Store prompts in `templates/propose.md.j2` and `templates/apply.md.j2`.
*   **Benefit:** You can iterate on prompts without recompiling.
*   **Benefit:** Users can override default prompts with their own templates via configuration.

### B. Direct LLM API Integration
**Current:** `tokio::process::Command::new("claude")`.
**Opportunity:** Replace with a direct HTTP client (using `reqwest`) or an SDK (like `async-openai` or a custom client).
*   **Benefit:** Removes the external CLI dependency.
*   **Benefit:** Enables **streaming responses**. You can show the user the agent's plan as it's being generated.
*   **Benefit:** Allows supporting multiple backends (OpenAI, Anthropic, local models) via configuration.

### C. Interactive "Human-in-the-Loop" Review
**Current:** `alc propose` generates files, then you run `alc fan-out`.
**Opportunity:** The tool creates `proposal.md` and `design.md`, but there is no step that *enforces* a human review before `fan-out`.
*   **Idea:** Add a `status` check that requires a human to "sign off" (e.g., `alc approve <change>`) before `fan-out` is allowed. This formalizes the "spec-driven" part of the workflow.

### D. Smart Context Selection
**Current:** Dumps all specs into the context.
**Opportunity:** Implement a simple heuristic or embedding-based search to only include relevant specs.
*   **Simpler Version:** Allow `registry.toml` to specify *which* specs a service cares about, or use the `capabilities` field to filter specs.

## 4. Code Specifics

**`src/git.rs`**
The credential handling is decent, but `credentials_callback` relies on `ssh_key_from_agent`. If a user uses a specific key file (not in the agent), this will fail. `git2` is notoriously finicky with SSH.
*   **Recommendation:** Fallback to shelling out to `git` CLI if `git2` fails. It's often more robust for auth because it uses the user's existing shell environment configuration perfectly.

**`src/main.rs`**
The `init_workspace` function creates a `registry.toml` template.
*   **Recommendation:** It should also create a `.gitignore` for the workspace that ignores `openspec/changes/*/` (except maybe `pipeline.toml`?) if the intention is *not* to commit generated artifacts. (Though usually, you *do* want to commit the plan).

## Summary Recommendation

To prepare for "iterative enhancement," the immediate next step should be **refactoring `OpsxEngine` to load prompts from files**. This separates the "code" (orchestration) from the "logic" (prompt engineering), allowing them to evolve independently.
