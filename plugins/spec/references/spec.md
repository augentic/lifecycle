# Artifact: Specs

**Write to**: `.specify/changes/<name>/specs/<crate>/spec.md` (one per crate)

## Template

```markdown
## ADDED Requirements

### Requirement: <!-- requirement name -->
<!-- requirement text -->

#### Scenario: <!-- scenario name -->
- **WHEN** <!-- condition -->
- **THEN** <!-- expected outcome -->
```

## Instruction

Create specification files that define WHAT the system should do.

First, read the proposal's **Source** section to determine the workflow:

---

**RT path** (Source is a repository URL):

  1. Clone the source repository. Invoke `/rt:git-cloner` with
     arguments:
       <repo-url> legacy/ true
     This clones the repo into `legacy/<repo-name>` as a detached tree.
  2. Generate specs and design. Invoke `/rt:code-analyzer` with
     arguments:
       legacy/<repo-name> <change-dir>
     code-analyzer produces both `specs/` and `design.md` in a single
     pass.
  3. Review the generated specs for completeness and adjust if needed.
  4. Proceed to the next artifact. design.md was already produced by
     code-analyzer — the design phase will review/enrich it.

---

**Omnia path** (Source is a JIRA/ADO/Linear epic key):

  Prerequisite: The Omnia plugin must be loaded for JIRA MCP access.

  1. Generate specs and design. Invoke `/plan:epic-analyzer` with
     arguments:
       <epic-key> <change-dir>
     epic-analyzer produces `proposal.md`, `specs/`, and `design.md`.
  2. Review the generated specs for completeness and adjust if needed.
  3. Proceed to the next artifact. design.md was already produced by
     epic-analyzer — the design phase will review/enrich it.

---

**Manual path** (Source is "Manual" or absent):

  Create one spec file per crate listed in the proposal's
  Crates section.

  Guidelines:
    - New crates: use the exact kebab-case name from the proposal
    (specs/<crate>/spec.md).
    - Modified crates: use the existing spec folder name from
    .specify/specs/<crate>/ when creating the delta spec at
    specs/<crate>/spec.md.

  Delta operations (use ## headers):
    - **ADDED Requirements**: New crates
    - **MODIFIED Requirements**: Changed behavior - MUST include full 
      updated content.
    - **REMOVED Requirements**: Deprecated features - MUST include 
      **Reason** and **Migration**.
    - **RENAMED Requirements**: Name changes only - use FROM:/TO: format

  Format requirements:
    - Each requirement: `### Requirement: <name>` followed by description
    - Use SHALL/MUST for normative requirements (avoid should/may)
    - Each scenario: `#### Scenario: <name>` with WHEN/THEN format
    - **CRITICAL**: In delta specs (this template), scenarios MUST use
      exactly 4 hashtags (`####`). In full baseline specs organized
      under `## Handler:` sections (see references/specify.md),
      scenarios are one level deeper at `#####`. Using fewer hashtags
      or bullets will fail silently.
    - Every requirement MUST have at least one scenario.

  MODIFIED requirements workflow:
    1. Locate the existing requirement in 
      .specify/specs/<capability>/spec.md
    2. Copy the ENTIRE requirement block (from `### Requirement:`
      through all scenarios).
    3. Paste under `## MODIFIED Requirements` and edit to reflect new 
      behavior.
    4. Ensure header text matches exactly (whitespace-insensitive)

  Common pitfall: Using MODIFIED with partial content loses detail at
  archive time.

  If adding new concerns without changing existing behavior, use ADDED
  instead.

  Example:
    
  ```markdown
  ## ADDED Requirements

  ### Requirement: User can export data
  The system SHALL allow users to export their data in CSV format.

  #### Scenario: Successful export
  - **WHEN** user clicks "Export" button
  - **THEN** system downloads a CSV file with all user data

  ## REMOVED Requirements

  ### Requirement: Legacy export
  **Reason**: Replaced by new export system
  **Migration**: Use new export endpoint at /api/v2/export
  ```

  Specs should be testable - each scenario is a potential test case.
