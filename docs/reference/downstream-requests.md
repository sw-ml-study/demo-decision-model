# Requests from projects that depend on this one

Nothing here authorizes work by itself: an item becomes a saga step when it is
revalidated against this repository's own plan. It exists so that an agent
working here can see what a downstream project is waiting on without someone
relaying it by hand.

The authoritative text of each ask lives in the asking project. This page
records what was asked, what was delivered, and where the answer is.

## sw-atlas (`software-wrighter-lab/sw-atlas`)

Builds a "hybrid docent": a deterministic keyword matcher proposes candidate
resources from a catalog of roughly 570 (blog posts, campus places,
repositories, videos), a typed decision model decides intent, the kind of
resource wanted and a few Nouls, and ordinary code arbitrates and quotes
catalog text into fixed frames. Nothing is generated. It vendors `lib/`
hash-pinned and pins the Rust crates; it edits nothing here. Its own copy of
these asks is `docs/demo-decision-model-requests.md` in that repository.

### TAG — a revision to pin — **delivered**

Asked for a git tag at which `crates/tdm-model` and `crates/tdm-trace` build as
standalone dependencies, with the bundle `version` stable inside a tag and the
parity tolerance recorded in the tag notes.

**`tdm-v0.1.0`**, at `b476842`. Verified by a consumer crate outside this
repository that depends on both crates by the tag alone, parses a committed
bundle, decides on it, and validates a committed trace. The contract a pin may
rely on is [`crate-contract.md`](crate-contract.md); the compatibility note that
matters today is that a version 3 bundle may now carry an empty
`keywords`/`match_order`, which readers built before `b13b1d0` reject.

### PR05 — dynamic choice sets — **delivered, with a result they should read
before building on it**

Asked for a question-conditioned scorer `score_i = f(h_state, h_question,
h_choice_i)` trained over `(state, question, choices, answer)` tuples, so one
set of weights scores choices it never saw, with the generality-cost comparison
against the fixed-head Choice; mlplunit-tested trainer and inference pair in
`lib/` with no demo identifiers, parity with the Rust forward pass, and measured
accuracy on choices held out of training.

Delivered as `lib/scorer.mlpl`, `crates/tdm-model/src/scorer.rs`
(`Scorer::rank` takes candidates as text at call time), tests in
`tests/test_scorer.mlpl` and `crates/tdm-model/tests/scorer_bundle.rs`, and the
lesson [`DC01`](../experiments/DC01-dynamic-choice-sets.md).

The measurement is the part to read. Where candidates trained, generality costs
about one point against the fixed head (85.3% against 86.1%). Where candidates
were **held out of training entirely**, the right card wins 0.4% of the time in
a full field and 36.5% in a five-card field — and against four equally untrained
cards, chance. On this corpus a card is a rule's pattern and replies, which
shares little but function words with an input, so this is a lower bound in an
unfriendly regime rather than a verdict on card scorers. It is not, however,
evidence for the claim that a resource published after training can be chosen
without retraining. That claim needs the same measurement on the sw-atlas
corpus, where a query and a card do share content words. The harness shape is
reusable, and the small all-cold field is the diagnostic that tells "weak" apart
from "biased against new cards".

### What sw-atlas offers back

- A second demo for the abstraction gate, over several hundred resources rather
  than demo 02's nine places: whatever forces a change to the vendored `lib/`
  comes back as a finding, with the diff. That is worth more to this repository
  than anything else on this page, because `NV01` exists precisely to test
  whether these abstractions are real or ELIZA-shaped.
- Frozen, hashed evaluation sets (paraphrase, off-topic, meta, follow-up) and
  scored rows for matcher alone, model alone, hybrid and ablations, with paired
  intervals on every margin.
