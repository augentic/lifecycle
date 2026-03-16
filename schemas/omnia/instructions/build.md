Implement the tasks in tasks.md. Do not write Rust code directly --
delegate to the skills below.

Arguments (used by all skills):
- CHANGE_ID: the name of this change (from specify status)
- PROJECT_DIR: the project root (typically ".")
- CRATE_NAME: the spec folder name (specs/<crate>/spec.md)
- CRATE_PATH: PROJECT_DIR/crates/CRATE_NAME

## Mode detection

Check whether CRATE_PATH/Cargo.toml exists:

- If Cargo.toml does not exist, use create mode.
- If Cargo.toml exists, use update mode.

---

## Create mode (Cargo.toml does NOT exist -- new crate)

### Phase 1: Generate

1. /omnia:guest-writer -- generate the WASM guest project
2. /omnia:crate-writer -- generate the domain crate (code only, no tests)
3. /omnia:test-writer -- generate comprehensive test suites from spec scenarios

Run each skill from start to finish. Complete every step in each skill's
process and ensure its verification checklist is satisfied before moving
to the next skill.

### Phase 2: Verify and repair

Run the verify-repair loop described below.

### Phase 3: Review (optional)

4. /omnia:code-reviewer -- AI code review

---

## Update mode (Cargo.toml exists -- incremental change)

### Step 0: Capture baseline

Before any changes, record the current test state:

```bash
cd $CRATE_PATH && cargo test 2>&1 | tee /tmp/$CRATE_NAME-baseline.txt
```

Record which tests pass and which fail. This baseline is used in
Phase 2 to detect regressions.

### Phase 1: Generate

1. /omnia:crate-writer -- update the domain crate (code only)
2. /omnia:test-writer -- update tests to match changed specs

Run each skill from start to finish. Complete every step in each skill's
process and ensure its verification checklist is satisfied before moving
to the next skill.

### Phase 2: Verify and repair

Run the verify-repair loop described below. In update mode, step 3
includes a regression check: compare post-test results against the
baseline captured in Step 0. Tests that passed before must still pass.

### Phase 3: Review (optional)

3. /omnia:code-reviewer -- AI code review

---

## Verify-repair loop (max 3 iterations)

After both crate-writer and test-writer have completed, run this loop
to converge on a clean build. Each iteration runs all three checks; if
any fail, apply the targeted fix and start a new iteration.

### 1. Formatting

```bash
cd $CRATE_PATH && cargo fmt --check
```

If fails: run `cargo fmt` to fix. Formatting is mechanical; one pass
suffices.

### 2. Compilation and lint

```bash
cd $CRATE_PATH && cargo check
cd $CRATE_PATH && cargo clippy -- -D warnings
```

If fails: fix each error or warning. Reference
[repair-patterns.md](../../../plugins/omnia/skills/crate-writer/references/repair-patterns.md)
for canonical Omnia SDK patterns (Handler structure, error handling,
serde conventions, clippy fixes).

### 3. Test suite

```bash
cd $CRATE_PATH && cargo test
```

If failures are detected, classify each failure and route the fix to
the appropriate skill:

| Failure signal | Classification | Fix action |
| --- | --- | --- |
| Error in `tests/` file paths, `MockProvider`, or `provider.rs` | **Test issue** | Re-enter test-writer with the error output |
| Error in `src/` file paths, missing trait impls in production code | **Code issue** | Re-enter crate-writer with the error output |
| Assertion mismatch where the *actual* value looks correct per spec | **Test issue** | Re-enter test-writer -- the expected value is wrong |
| Assertion mismatch where the *expected* value matches spec | **Code issue** | Re-enter crate-writer -- the handler returns the wrong result |
| MockProvider missing a trait impl the handler now requires | **Test issue** | Re-enter test-writer to update MockProvider |
| Type mismatch between handler output and test assertion | **Code issue** if handler type is wrong per spec; **test issue** if assertion type is stale | Classify per spec, fix accordingly |

When re-entering a skill for repair, pass the full error output as
context so the skill can make a targeted fix. Reference
[mock-provider.md](../../../plugins/omnia/skills/test-writer/references/mock-provider.md)
for test-side repair strategies.

**Update mode only**: compare post-test results against the baseline
from Step 0. Tests that passed before and now fail are regressions and
must be fixed before proceeding.

### Loop control

Repeat from step 1 until all three checks pass or 3 iterations are
exhausted. If still failing after 3 iterations: **STOP**. Do not mark
the task complete. Report the remaining failures with full error output
and escalate for guidance.
