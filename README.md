# Augentic Delivery Lifecycle

A custom [OpenSpec](https://github.com/Fission-AI/OpenSpec/) schema that defines how Augentic plans, specifies, and delivers changes to code built using Augentic plugins.

## What This Is

OpenSpec provides spec-driven development for AI coding assistants — you agree on *what* to build before any code gets written. This repo contains Augentic's custom schema (`augentic`), tailored for the constraints of `wasm32-wasip2` targets, the `Handler<P>` pattern, and provider-based I/O.

Where OpenSpec's built-in `spec-driven` schema is general-purpose, this schema adds:

- **Current-state analysis** as a first-class artifact — the AI reads existing crate source before proposing anything
- **Change classification** (additive / subtractive / modifying / structural) with ordered execution
- **Pipeline configuration** that maps changes to crates and declares cross-target dependencies
- **Omnia-aware context and rules** injected into every artifact via `config.yaml`

## Artifact Pipeline

The schema defines six artifacts with explicit dependencies:

```
current-state ──→ proposal ──→ specs ──┐
                     │                  │
                     └──→ design ───────┤
                                        ▼
                                    manifest ──→ pipeline-config
```


| Artifact            | Purpose                                                                                   |
| ------------------- | ----------------------------------------------------------------------------------------- |
| **current-state**   | Inventory of handlers, types, provider usage, tests, and dependencies in affected crates  |
| **proposal**        | Why this change is needed, what's in scope, and what risks exist                          |
| **specs**           | BDD specifications per capability with `+`/`-` deltas against existing specs              |
| **design**          | Domain model, API contracts, provider changes, structural changes — with mermaid diagrams |
| **manifest**        | Classified, ordered change plan with complexity assessment and affected file predictions  |
| **pipeline-config** | TOML execution config mapping targets to crates, specs, and routes                        |


## Repo Structure

```
├── config.yaml              # Project config: schema default, platform context, per-artifact rules
├── schemas/
│   └── schema.yaml          # Artifact DAG and dependency graph
└── templates/
    ├── current-state.md     # Template: crate inventory extraction
    ├── proposal.md          # Template: change proposal
    ├── specs.md             # Template: BDD specifications
    ├── design.md            # Template: technical design
    ├── manifest.md          # Template: classified change plan
    └── pipeline.md          # Template: pipeline TOML generation
```

### `config.yaml`

Sets `augentic` as the default schema and injects platform context (Rust WASM, Omnia SDK, Handler pattern, provider capabilities) into every artifact. Per-artifact rules enforce conventions — for example, specs must use Given/When/Then format and current-state must extract handler trait bounds.

See the OpenSpec [customization docs](https://github.com/Fission-AI/OpenSpec/blob/main/docs/customization.md) for how config, context, and rules are resolved and injected.

### `schemas/schema.yaml`

Defines the six artifacts and their dependency graph. OpenSpec enforces this ordering — you can't create a manifest until specs, design, and current-state all exist.

### `templates/`

Markdown templates injected into the AI prompt when generating each artifact. Each template contains instructions specific to Augentic's codebase patterns (handler inventory format, serde attribute tracking, provider capability extraction, change classification decision trees).

## Usage

With [OpenSpec installed](https://github.com/Fission-AI/OpenSpec/#quick-start), point it at a project that contains this schema:

```bash
# Propose a change (current-state is generated first)
/opsx:propose "add webhook delivery handler"

# Generate specs and design from the approved proposal
/opsx:continue

# Generate manifest and pipeline config
/opsx:continue

# Implement
/opsx:apply
```

The `config.yaml` context ensures the AI knows about WASM constraints, the Handler pattern, and provider capabilities throughout the entire workflow.

## Customization

To adapt this schema for a different Augentic project, edit `config.yaml` to update the platform context. To change the artifact workflow itself, edit `schemas/schema.yaml` — for example, adding a `review` artifact between `design` and `manifest`.

See the OpenSpec [customization guide](https://github.com/Fission-AI/OpenSpec/blob/main/docs/customization.md) for the full reference on schemas, templates, and config resolution.