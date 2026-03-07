# Lifecycle Repo Review

## Recommendation

Keep the system intentionally narrower than it currently claims to be.

The repo is already small and readable, which is a strength. The main missed opportunity is that the design advertises more flexibility than the implementation actually supports. That extra surface area makes the tool harder to trust, harder to explain, and harder to extend safely.

The simplest path forward is:

1. Pick one orchestration unit and use it consistently end to end
2. Remove or defer knobs that are parsed but not operational
3. Either make the engine boundary real, or describe the tool as OPSX-specific for now

## What Is Working Well

- The codebase is compact and easy to scan.
- Core concepts are separated into sensible modules: `registry`, `pipeline`, `status`, `git`, `agent`.
- The CLI workflow is easy to explain: `propose -> fan-out -> apply -> sync -> archive`.
- The repo-level grouping idea is good and matches how users think about branches and PRs.
- The `pipeline` and `status` modules are the clearest parts of the design and form a decent base for future refinement.

## Main Findings

### 1. The execution unit is inconsistent

`fan_out` works at the repo-group level, but `apply` works at the target level.

That mismatch creates avoidable complexity:

- one repo can be cloned multiple times during apply
- one branch can receive multiple repo-wide apply passes
- retries and idempotence become harder to reason about
- status becomes noisier than the actual GitHub workflow

This is the largest simplification opportunity in the repo.

If a shared repo gets one branch and one PR, it should usually also get one apply execution. Within that execution, the agent can still be told which services or crates are affected.

### 2. The model promises monorepo flexibility that the runtime does not really support

The pipeline and registry both carry `project_dir`, per-target overrides, and branch information. The README also documents those fields as meaningful. But the operational flow mostly clones the repo and runs from repo root.

This creates a dangerous kind of complexity: conceptual complexity without corresponding behavior.

Today the code seems best described as:

- repo-level orchestration
- root-level checkout and execution
- optional per-target branch selection

That is a valid MVP. The problem is pretending it is already a more general execution framework.

### 3. Status tracking is finer grained than the delivery mechanism

The real delivery artifact is one PR per repo group, but status is tracked per target and the same PR URL is copied onto multiple targets.

That means:

- review state is duplicated
- sync repeats PR checks that refer to the same underlying object
- progress reporting reflects internal modeling more than user reality

For usability, a repo-level status model would likely be simpler:

- repo group: branch, PR, review, merge, archive readiness
- optional nested target detail for planning only

### 4. The engine abstraction is only partial

The code comments and README describe the orchestrator as engine-agnostic, but the implementation is still strongly OPSX-shaped:

- `main.rs` hardwires `OpsxEngine`
- `sync.rs` hardcodes `openspec/changes`
- agent execution is tied to `OPSX_AGENT_BACKEND`
- CLI behavior and prompts still assume OPSX semantics throughout

This is not inherently bad. The issue is architectural ambiguity.

A simpler and clearer choice would be one of these:

1. Say explicitly that this tool is OPSX-first and keep the design concrete
2. Finish pushing all path, artifact, and command assumptions behind the engine trait

Right now it sits awkwardly between those two positions.

### 5. The system exposes more configuration than it currently uses

There are several examples of this:

- `concurrency` exists in `pipeline.toml` but is not used
- both inline `depends_on` and rich `[[dependencies]]` exist, even though only edge direction seems operational
- `Archived` exists in the state machine, but archive just moves the change folder
- multiple spec path shapes are accepted, but brief generation normalizes to only one path convention

Each of these is small, but together they make the tool feel more abstract and less predictable.

The repo would be easier to understand if unsupported or inactive options were removed until needed.

### 6. `propose` will get more expensive and less understandable as the registry grows

The current approach gathers registry info, then walks every unique repo and appends all spec Markdown it can find into a single context blob.

That is attractive in an MVP because it avoids pre-analysis, but it has clear scaling costs:

- longer and noisier prompts
- slower propose runs
- less explainable impact analysis
- more ways for irrelevant context to distort output

For iterative enhancement, a staged approach would likely age better:

1. identify candidate services from registry and description
2. load only their local specs and nearby context
3. generate the change artifacts from that narrower set

## Design Simplifications I Would Make

### Simplification 1: Make repo group the operational unit

Use the same unit through:

- fan-out
- apply
- sync
- archive readiness

Targets can remain in the planning artifact, but execution should operate on repo groups.

### Simplification 2: Reduce the public data model to the behavior you actually support

For example, keep only:

- `id`
- `repo`
- `crate`
- `capabilities`
- `depends_on`
- optional `branch`

Then add `project_dir`, concurrency, or richer dependency metadata only when they become active in the runtime.

### Simplification 3: Collapse duplicated state

Prefer:

- one repo-group status entry per PR-bearing unit
- target-level details only where they materially affect execution order

This would make `sync` and `status` outputs easier for users to interpret.

### Simplification 4: Choose between “extensible” and “concrete”

If extensibility matters now:

- make the engine trait the true boundary
- remove hardcoded OPSX paths from orchestration code
- make backend selection engine-owned or command-owned, not globally OPSX-named

If extensibility can wait:

- rename concepts more concretely
- remove the engine abstraction for now
- reintroduce it later once a second engine exists

The latter may actually be the better MVP.

### Simplification 5: Separate planning richness from execution strictness

It is fine for `pipeline.toml` to contain rich metadata for reviewers and future tooling. But the execution engine should consume a smaller, stricter subset that is unambiguous and fully implemented.

That gives you room to evolve planning artifacts without making the runtime harder to reason about.

## Specific Implementation Risks

### Apply can report success after a failed push

If the agent succeeds but `git add/commit/push` fails for a real reason, the target can still move to `implemented`.

That is a correctness problem, not just a polish issue, because later commands trust that status.

### Shared-repo apply likely repeats work

Because the brief is generated from the repo group but invoked per target, multiple targets in the same repo are likely to receive overlapping instructions.

This increases the chance of:

- duplicate changes
- confusing agent behavior
- no-op commits mixed with real failures

### `project_dir` is present but mostly inert

This is exactly the kind of field that makes future enhancement harder because users will naturally start depending on it before it is truly supported.

## Documentation Drift

There are already signs of conceptual drift:

- README contains command typos like `lc apply`
- `CONTRIBUTING.md` still refers to `omnia`
- design documents and the current code disagree in places about how much automation or abstraction exists

That drift matters because this tool's value is mostly operational clarity. If users cannot tell what is aspirational versus what is implemented, they will form the wrong mental model very quickly.

## Testing Gaps

The test coverage is strongest in the pure data modules:

- `pipeline`
- `status`
- `registry`
- `brief`

The biggest untested areas are the orchestration edges:

- shared-repo multi-target apply
- push failure handling
- branch and override behavior
- repeated sync against the same PR
- archive behavior versus status semantics

Those are also the places where the design is currently most fragile.

## Suggested Near-Term Roadmap

### Phase 1: Make the MVP honest

- Treat repo group as the apply unit
- Fail apply if commit or push fails, unless the no-op case is detected explicitly
- Remove or hide inactive fields from README and examples
- Fix documentation drift and command typos

### Phase 2: Simplify the status model

- Introduce repo-group level status as the primary operational state
- Keep target details as planning metadata, not the main runtime object
- Make sync reason in terms of PR-bearing units

### Phase 3: Decide the abstraction strategy

Choose one:

1. Commit to OPSX-specific implementation and simplify the code accordingly
2. Commit to multi-engine support and finish the boundary cleanly

Either is better than the current half-abstract position.

## Bottom Line

This repo is already close to a good MVP because it is small and conceptually focused. The main problem is not excessive code. It is excess implied generality.

The best improvement is to make the implementation more opinionated:

- one execution unit
- one clear status model
- one honest abstraction boundary
- fewer inactive knobs

That would make the tool easier to use immediately and much easier to evolve without accumulating accidental complexity.
