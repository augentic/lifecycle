# Lifecycle Repo Review (GPT-5.3 Codex)

## Design and Implementation Critique

This review focuses on missed opportunities to make the system simpler to use, easier to understand, and better prepared for iterative enhancement.

## Critical Issues

### 1) False "implemented" status on failed push

In `apply`, a failure from `git::add_commit_push()` is logged as a warning, but the target still transitions to `implemented`. This can misrepresent reality by showing successful implementation when nothing was pushed.

**Why this matters**
- Breaks trust in `status.toml` as source of truth
- Makes retries and debugging confusing
- Can advance downstream targets on false completion

## Important Simplicity Risks

### 2) Mixed execution model: repo-grouped fan-out vs target-loop apply

`fan-out` operates per repo group (one branch/PR per repo), while `apply` loops per target and repeatedly invokes the agent for the same repo group context.

**Impact**
- Duplicate work for multi-target repos
- Higher chance of overlapping commits or nondeterministic outcomes
- More complex mental model for users ("is execution per repo or per target?")

### 3) Engine abstraction leakage

Most paths are engine-derived, but `sync` hardcodes `openspec/changes`. That weakens the `Engine` abstraction and makes future engine adapters harder than necessary.

**Impact**
- Hidden coupling to OPSX path conventions
- Inconsistent behavior if new engines are introduced

### 4) Config surface larger than behavior

`Pipeline` includes fields like `concurrency` and models `project_dir`, but orchestration does not use them meaningfully.

**Impact**
- Users assume capabilities that are not implemented
- Adds cognitive load to docs and pipeline authoring

### 5) Naming drift in UX/docs (`alc`, `lc`, `opsx`)

CLI branding and command text are inconsistent across docs and runtime messages.

**Impact**
- Lowers usability and trust
- Increases onboarding friction

## Architectural Missed Opportunities

### A) Make one execution unit canonical

Pick one of:
- **Repo-first execution** (recommended): fan-out/apply/status all operate on repo groups
- **Target-first execution**: one branch/PR per target (less aligned with current design)

Current behavior combines both and inherits complexity from each.

### B) Clarify state semantics

`implemented` should mean one concrete thing. Recommended:
- `implemented` = remote branch updated and push succeeded
- Optional additional state if needed: `implemented_local`

### C) Strengthen module boundaries

`main` currently handles part of command wiring/state load logic directly. A clearer command boundary would make behavior easier to test and evolve.

### D) Consolidate design documentation

There are multiple proposal generations (`v1`, `v2`, `v3`, `next`) with overlapping concepts. This is useful history, but for day-to-day use it increases ambiguity about canonical behavior.

## Testing Gaps

Current tests are strong in domain units (`pipeline`, `status`, `registry`, `brief`) but weak in end-to-end orchestration behavior.

### Missing high-value tests
- Full lifecycle flow (`propose -> fan-out -> apply -> sync -> archive`)
- Push/commit failure behavior (must not become `implemented`)
- Multi-target single-repo apply behavior
- Resume/idempotency behavior after partial failure

## Prioritized Improvements

### P0 (correctness + trust)
- Make push/commit failure block `implemented` transition
- Add explicit failure reason capture in status

### P1 (simplify mental model)
- Unify on repo-group execution semantics in `apply`
- Update status/reporting to reflect repo and target views cleanly

### P2 (future flexibility)
- Route all path conventions through `Engine`
- Remove unused pipeline fields or implement them fully

### P3 (developer experience)
- Standardize naming/terminology across CLI, logs, README, and docs
- Publish one canonical architecture doc + mark older docs as historical

## Suggested Iterative Enhancement Plan

1. **Stabilize state truthfulness**: correctness fixes first
2. **Normalize execution granularity**: repo-first workflow end to end
3. **Harden abstraction seams**: engine-path consistency
4. **Tighten public contract**: config/docs aligned to implemented behavior
5. **Add orchestration integration tests**: protect future iteration speed

This sequence minimizes rework and gives immediate user-facing reliability improvements while preserving the current architecture direction.

