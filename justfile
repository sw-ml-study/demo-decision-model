set shell := ["sh", "-cu"]

# Show available repository tasks.
default:
    @just --list

# The full pre-commit gate: structure, boundaries, links, catalog, style, tests, probes.
check:
    ./scripts/check

# Run native mlplunit tests; arguments select paths, tags, or filters.
tests *args:
    ./scripts/run-tests {{args}}

# Re-run the upstream-finding reproducers and require the documented outcome.
probes:
    ./scripts/run-probes

# Check canonical formatting and the module-purpose comment on tracked MLPL source.
mlpl-style:
    ./scripts/check-mlpl-style

# Check that no demo vocabulary has leaked into lib/, crates/, or schemas/.
abstraction:
    ./scripts/check-abstraction

# Check that every catalog entry names files that exist.
catalog:
    ./scripts/check-catalog

# Check that every relative documentation link resolves.
doc-links:
    ./scripts/check-doc-links

# Rust gate for the microscope crates: formatting, pedantic clippy, and tests.
rust:
    ./scripts/check-rust

# Validate every committed decision trace against decision-trace-v1.
traces:
    ./scripts/check-traces
