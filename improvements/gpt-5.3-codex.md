# Repository Design & Implementation Critique

## Scope

Review of architecture and implementation with focus on simplification, understandability, and readiness for iterative enhancement.

## Findings (Ordered by severity)

### High

1. `apply --target` can bypass dependency safety for rich `[[dependencies]]` edges.
   - Target filtering drops unrelated levels, and runtime blocking checks only inline `depends_on`.
   - Result: downstream targets may run early if upstream edges are expressed only in `[[dependencies]]`.

2. Apply flow can mark a target `implemented` even when commit/push fails.
   - `git::add_commit_push` errors are logged and ignored as warnings.
   - Result: status can indicate success while code/branch state is stale or unchanged.

### Medium

3. `project_dir` is modeled but not used end-to-end.
   - Present in registry/pipeline/grouping, but execution paths do not route work through this subdirectory.
   - Result: apparent monorepo/subdir support that is not reliably enforced in behavior.

4. Repo grouping has hard constraints that reduce adoption in realistic monorepos.
   - Shared-repo targets must share one `domain` and one `project_dir`, or fan-out fails.
   - Result: brittle behavior for mixed-domain or multi-subtree repositories.

5. Branch override behavior is implicit and potentially lossy.
   - Group branch derives from the first target only; conflicting branch overrides are not surfaced.
   - Result: hard-to-predict execution and potential branch mistakes.

### Low

6. Documentation and implementation drift on GitHub integration.
   - README describes `gh`-driven PR operations, while implementation uses `octocrab` and `GITHUB_TOKEN`.
   - Result: avoidable setup confusion and onboarding friction.

## Missed Opportunities to Simplify

- Use one dependency model (`all_edges`) everywhere:
  - sorting, filtering, blocking checks, and status gating.
- Make apply outcomes explicit:
  - distinguish "implemented with pushed changes" vs "implemented with no changes" vs "failed to push".
- Either fully implement `project_dir` or remove it until supported.
- Validate branch consistency within grouped targets early and fail clearly on conflicts.
- Align README and runtime requirements to one authoritative GitHub/auth path.

## Iterative Enhancement Recommendations

1. **Dependency correctness first**
   - Refactor blocker logic to read from the same edge source as topological sort.
   - Add regression tests for `--target` with mixed edge styles.

2. **State truthfulness**
   - Treat push/commit failure as a failure state unless an explicit "no changes" signal is detected.
   - Add status semantics for "no-op implementation" if desired.

3. **Monorepo support clarity**
   - Decide if `project_dir` is required behavior now.
   - If yes, thread it through fan-out/apply/brief/agent command context.
   - If no, remove from user-facing config and docs for now.

4. **Configuration ergonomics**
   - Introduce stronger upfront validation (conflicting branch overrides, mixed grouping assumptions).
   - Keep errors actionable with exact target IDs and expected fixes.

5. **Doc/UX coherence**
   - Synchronize README with actual API client behavior and required env vars.
   - Prefer one canonical auth path and mention alternatives only if supported in code.

## What Is Already Strong

- Clear command lifecycle and mental model (`propose -> fan-out -> apply -> sync -> archive`).
- Good separation of concerns (`pipeline`, `status`, `git`, `engine`, `context`).
- `ChangeContext` reduces command boilerplate and centralizes validation/loading.
- Pipeline graph utilities and state-transition checks provide a solid foundation.

## Suggested Next Review Pass

- Stress-test failure handling and re-run behavior with partial failures.
- Validate behavior in shared-repo, multi-crate, multi-domain monorepo scenarios.
- Expand integration tests around apply/sync status transitions to protect iterative refactors.
