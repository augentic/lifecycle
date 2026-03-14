# Check Type Definitions

Structured check types used by `validate_checks` and `cross_artifact_checks`
in `schema.yaml`. Skills that run or reference these checks (review, propose)
should consult this document for semantics and parameters.

## Per-Artifact Check Types

These checks run against individual artifact files.

**`heading_exists`** -- Verify the heading exists and has content below it.
- `heading`: the markdown heading to find (exact match)
- `min_content_lines`: minimum non-empty lines between this heading and the next heading (default 1)
- `content_matches` (optional): regex pattern that must match at least one line in the section

**`heading_exists_or_waived`** -- Like `heading_exists`, but the section may contain a waiver instead of content.
- `heading`: the markdown heading to find
- `waiver_pattern`: regex that, if matched in the section, counts as a valid waiver

**`spec_structure`** -- Validate that spec files follow the heading hierarchy from `spec_format`.
- `heading_ref`: reference to `spec_format.requirement_heading`
- `id_ref`: reference to `spec_format.requirement_id_prefix`
- `scenario_ref`: reference to `spec_format.scenario_heading`
- For each requirement heading: verify an ID line follows immediately, then at least one scenario heading exists within the requirement block.

**`requirement_has_id`** -- Every requirement heading must be followed by a line starting with the ID prefix, matching the pattern.
- `pattern_ref`: reference to `spec_format.requirement_id_pattern`

**`requirement_has_scenario`** -- Every requirement block must contain at least one scenario heading.
- `scenario_ref`: reference to `spec_format.scenario_heading`

**`normative_language`** -- Requirement text must use normative terms.
- `required_terms`: at least one of these must appear in each requirement's description text
- `forbidden_as_normative`: these terms must not be used as normative language (informational use is acceptable)

**`scenario_keywords`** -- Scenario content must include the required keywords.
- `required`: list of keywords (e.g., `["WHEN", "THEN"]`) — each must appear in the scenario block

**`pattern_match`** -- Lines in a specific scope must match a regex.
- `scope`: which lines to check (e.g., `task_lines`, `crate_names`, `capability_names`)
- `pattern`: regex each matching line must satisfy

**`heading_match`** -- The file must contain headings matching a pattern.
- `pattern`: regex for heading lines
- `min_count`: minimum number of matching headings

**`min_count`** -- Minimum number of lines matching a pattern in a scope.
- `scope`: which lines to check
- `pattern`: regex to match
- `min`: minimum count

## Cross-Artifact Check Types

These checks validate consistency across multiple artifacts.

**`proposal_crates_have_specs`** / **`proposal_capabilities_have_specs`** -- For every crate or capability listed in the proposal (under New or Modified headings), verify a corresponding spec file exists at `specs/<name>/spec.md` in the change directory. Report any crates/capabilities without specs.

**`design_references_valid`** -- Scan `design.md` for requirement ID references matching the `spec_format.requirement_id_pattern` (e.g., `REQ-001`). For each referenced ID, verify it exists in one of the spec files. Report any orphaned references.

**`spec_format_valid`** -- For every spec file in the change, validate the complete heading structure:
- Every `### Requirement:` heading is followed by an `ID:` line
- The ID matches `spec_format.requirement_id_pattern`
- At least one `#### Scenario:` exists within each requirement block
- No content appears outside of recognized sections
