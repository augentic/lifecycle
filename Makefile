.PHONY: checks
checks:
	@./scripts/checks.sh

.PHONY: merge-specs
merge-specs:
	@python3 scripts/merge-specs.py $(ARGS)
