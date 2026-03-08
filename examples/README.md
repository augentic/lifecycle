# Examples

## Planning and execution directories

Demonstrates the core planning-to-execution artefact structure without git, GitHub, or an AI agent. Two directories model the split:

- **`planning/`** -- a central workspace with `registry.toml` and change artefacts under `openspec/changes/add-caching/`
- **`repo/`** -- a target repository showing what lands after `alc apply` distributes specs

### Directory layout

```
examples/
  planning/                              # central planning workspace
    registry.toml                        # service catalogue (2 services, same repo)
    openspec/
      changes/
        add-caching/
          proposal.md                    # why + what
          design.md                      # technical design per service
          tasks.md                       # implementation tasks grouped by repo
          pipeline.toml                  # targets, dependencies, execution config
          specs/
            cache-api/
              spec.md                    # delta spec for cache-service
            gateway-cache-integration/
              spec.md                    # delta spec for api-gateway
  repo/                                  # target repo after distribution
    .opsx/
      brief.toml                         # per-repo change brief
      specs/                             # distributed delta specs
        cache-api/spec.md
        gateway-cache-integration/spec.md
      upstream/                          # upstream context from central plan
        design.md
        tasks.md
        pipeline.toml
```

### How to read this

Start with `planning/` -- this is what `alc propose` generates:

1. **`registry.toml`** -- the service catalogue listing every target service, its repo, domain, and capabilities
2. **`openspec/changes/add-caching/proposal.md`** -- the change proposal (why and what)
3. **`design.md`** and **`tasks.md`** -- technical design and implementation tasks per service
4. **`pipeline.toml`** -- execution config: which targets to implement, in what order, with what dependencies
5. **`specs/`** -- delta specs describing what changes in each service's API or behaviour

Then look at `repo/` -- this is what each target repo receives when `alc apply` runs distribution:

1. **`.opsx/brief.toml`** -- a summary tying the repo to the change, listing its crates, specs, and upstream context paths
2. **`.opsx/specs/`** -- the delta specs relevant to this repo, copied from the central plan
3. **`.opsx/upstream/`** -- the design, tasks, and pipeline from the central plan for context

### Relationship to the full CLI

The `planning/` directory mirrors what lives in your planning repo after `alc propose`. The `repo/` directory mirrors what `alc apply` distributes into each target repo before invoking the AI agent. The data flow in the real CLI is:

```
planning/registry.toml  ──┐
planning/openspec/...   ──┤──▶ Registry::load + Pipeline::load
                          │──▶ Pipeline::validate
                          │──▶ Pipeline::group_by_repo
                          └──▶ OpsxEngine::distribute ──▶ repo/.opsx/
```

### Editing fixtures

The fixture files under `planning/` are plain TOML and Markdown. Edit them to experiment with different registry shapes, pipeline topologies, or spec content.
