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

# Re-export every literate document under docs/literate/ to its published HTML (Emacs, batch, nothing evaluated).
literate:
    ./scripts/publish-literate

# Check every literate document: tangle parity, primer results, and a fresh export.
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

# Featurize demo 01's oracle corpus for model 4 (Rust; writes tmp/model4/data.json).
model4-featurize:
    mkdir -p tmp/model4
    cargo run --release -p eliza-sim --bin featurize -- demos/eliza/oracle/corpus.tsv demos/eliza/oracle/frame-nouls.tsv demos/eliza/probe-labels.tsv demos/eliza/probe-nouls.tsv fixtures/bundles/demo01-doctor.json tmp/model4/data.json

# Train model 4 on the featurized corpus and rewrite its bundle (about 11 minutes).
model4-train: model4-featurize
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/eliza/v4/export.mlpl

# Score model 4 against the oracle: agreement, per rule, calibration, policy.
model4-score:
    cargo run --release -p eliza-sim --example oracle_score

# Rewrite the Noul labels of the generated sentences (demos/eliza/oracle/frame-nouls.tsv).
model4-frame-nouls:
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/eliza/v4/frame-nouls.mlpl

# Build the (state, question, choices, answer) tuples for the PR05 card scorer.
scorer-cards:
    mkdir -p tmp/model5
    cargo run --release -p eliza-sim --bin cards -- demos/eliza/oracle/corpus.tsv demos/eliza/oracle/frame-nouls.tsv fixtures/bundles/demo01-doctor.json tmp/model5/data.json

# Train the PR05 card scorer and rewrite its bundle (about 6 minutes).
scorer-train: scorer-cards
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/eliza/v5/export.mlpl

# Score the card scorer against model 4's fixed head, including on cards held out of training.
scorer-score:
    cargo run --release -p eliza-sim --example scorer_score

# Train demo 01's Choice model and rewrite its weights (about 25 seconds).
eliza-train:
    ./scripts/run-eliza-train

# Score local LLMs on the same held-out messages, by the same bounded-choice method (needs ollama).
llm-baseline:
    ./scripts/llm-baseline

# Run the concise hello: the short annotated program, over the corpus it is given.
concise-hello:
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/typed-decisions/concise-hello.mlpl -- demos/typed-decisions/messages.json

# Run the hello-world typed-decision demo (about three seconds).
typed-decisions:
    ../sw-mlpl/target/release/mlpl-repl --source-dir . -f demos/typed-decisions/typed-decisions.mlpl

# Bundle the in-browser training program and run it standalone, as the gate does.
browser-program:
    ./scripts/check-browser-program

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
