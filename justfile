set shell := ["sh", "-cu"]

# Show available repository tasks.
default:
    @just --list

# Talk to demo 01: `just chat "my mom never listens to me"`. Loads the committed weights.
chat text:
    @./scripts/run-chat "{{text}}"

# Run the committed demo transcript through demo 01, one turn per line.
transcript:
    @./scripts/run-chat < demos/eliza/transcript.txt

# Serve the browser demo locally with live rebuild at http://127.0.0.1:8080/.
web:
    cd crates/tdm-web && trunk serve --open

# Re-export the literate document to docs/literate/literate.html (Emacs, batch, nothing evaluated).
literate:
    ./scripts/publish-literate

# Check the literate document: tangle parity, primer results, and a fresh export.
tangle:
    ./scripts/check-tangle

# Build the live demo into pages/ (committed; the Pages workflow publishes it as is).
site:
    ./scripts/build-site

# Serve the built pages/ directory at http://127.0.0.1:8765/ exactly as Pages will.
serve-site:
    cd pages && python3 -m http.server 8765 --bind 127.0.0.1

# Re-export demo 01's trained model as the JSON bundle the browser embeds.
eliza-export:
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/eliza/export.mlpl

# Train demo 01's Choice model and rewrite its weights (about 25 seconds).
eliza-train:
    ./scripts/run-eliza-train

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
