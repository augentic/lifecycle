# Lifecycle Repo Review

## Summary

The repository has a good small-module shape and a clear happy-path workflow. `README.md` makes the intended lifecycle easy to follow, and the split between `propose`, `fan-out`, `apply`, `sync`, and `archive` keeps the codebase navigable.

The main design weakness is conceptual drift between the public model and the actual implementation. Several concepts look more general than the system really is today, which increases cognitive load and makes iterative enhancement riskier than it needs to be.

## Main Findings

### 1. `apply` can report success when the remote state is stale

In `src/apply.rs`, a successful agent run is treated as implementation success even if `git::add_commit_push()` fails. That means auth problems, push conflicts, or branch issues can leave the PR unchanged while `status.toml` records the target as `implemented`.

This is the highest-risk behavior mismatch in the repo because it weakens the reliability of the status model.

### 2. `project_dir` is modeled but not really honored

The schema and docs present `project_dir` as a first-class concept, which suggests support for monorepos or nested crates. In practice, distribution writes `.opsx` at the repo root and `apply` invokes the agent at the repo root as well.

That makes the mental model harder to trust:

- users can set `project_dir`
- grouping validates `project_dir`
- execution does not actually use it

Either remove this concept until it is real, or wire it through end-to-end.

### 3. State is persisted per target, but work happens per repo group

The runtime unit is a repo group: one checkout, one branch, one PR, one agent run. The persisted state unit is a target. That forces the code to spread repo-level events across multiple target records and duplicates PR/repo metadata across status entries.

This is workable for the current scope, but it is an awkward foundation for:

- partial retries
- richer PR metadata
- repo-level logs and diagnostics
- future review automation

The execution unit and persistence unit should match.

### 4. The `Engine` abstraction is broader than the current needs

There is only one engine implementation, but the code already pays the cost of a polymorphic abstraction across CLI parsing, dispatch, prompts, file layout, and archive naming.

This trait is not obviously wrong, but it appears early. At the current stage it adds indirection faster than it adds flexibility.

Missed simplification opportunity: keep `opsx` concrete until a second engine forces the seam, or narrow the abstraction to only the parts that genuinely vary.

### 5. `propose` gathers too much context too eagerly

`src/propose.rs` reads all services, deduplicates by repo, then loads every Markdown spec it can find from every referenced repo into a single prompt context.

That is simple to implement, but it will age poorly:

- prompt size grows with the registry rather than with change scope
- context relevance gets noisier over time
- failures in spec discovery are easy to miss
- the selection logic is hard to test independently

This should probably become a more explicit context-selection step instead of a full-registry scrape.

### 6. Dependencies have two overlapping representations

Execution edges can come from inline `depends_on` fields and from `[[dependencies]]`. The rich dependency model carries metadata, while the inline form only carries ordering.

That flexibility makes authoring easier in the short term, but it creates a double-entry model that future code has to merge, deduplicate, explain, and validate.

Missed simplification opportunity: pick one primary representation and derive the lighter-weight view from it.

## Design Critique

### What is already working well

- Command boundaries are clear and readable.
- `ChangeContext` is a good consolidation point.
- Pipeline grouping and graph logic are easy to locate.
- The README does a solid job explaining the happy path.

### Where the design feels misleading

- The public data model implies broader support than the runtime currently provides.
- Repo-group behavior is central, but target-oriented persistence dominates the state model.
- Engine pluggability is advertised in the architecture earlier than the rest of the system justifies.

These are not catastrophic issues, but they create friction for both users and future contributors because the simplest explanation of the system is not quite the true one.

## Recommendations

### Highest priority

1. Make `implemented` mean "agent succeeded and remote branch reflects the result".
2. Align the persisted state model with repo groups, branches, and PRs.
3. Either fully implement `project_dir` semantics or remove/defer it.

### Next simplifications

1. Collapse or narrow the `Engine` abstraction until a second engine exists.
2. Replace full-registry spec ingestion in `propose` with a scoped context selection step.
3. Standardize dependency declaration on one source of truth.

## Suggested Refactor Direction

The cleanest next step is to make the execution unit explicit as a repo-level work item:

- one work item per repo group
- branch, PR URL, and execution state stored once
- target IDs attached as metadata

That single shift would simplify `fan-out`, `apply`, `sync`, and status reporting together.

After that, the next best cleanup is to decide whether this tool is truly monorepo-aware. If yes, `project_dir` should shape distribution paths, agent invocation paths, and commit scope. If no, it should not be user-facing yet.

## Testing Gaps

The test coverage is strongest around parsing and graph/state helpers. The riskier workflow edges are less protected:

- `src/apply.rs`
- `src/fan_out.rs`
- `src/propose.rs`
- `src/sync.rs`
- `src/github.rs`
- `src/agent.rs`

Those modules contain the failure modes most likely to matter during iterative enhancement.
