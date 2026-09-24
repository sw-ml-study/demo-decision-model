# Demo 02: someone else's catalog

The programs here measure a downstream project's design against its own data,
and are the repository's `NV01` test: a second domain, with `lib/` unchanged.
What `lib/` needed in order to survive it is written up in
[`AT01`](../../docs/experiments/AT01-atlas-rerank.md) and was a real defect.

Nothing of sw-atlas's is committed here. The harness reads a checkout:

```
python3 demos/atlas/extract.py --atlas ../../software-wrighter-lab/sw-atlas
python3 demos/atlas/featurize.py          # cards, queries, splits, candidates
python3 demos/atlas/intent.py             # the question sets, for the other heads

mlpl-repl --source-dir . -f demos/atlas/train.mlpl  -- tmp/atlas/data.json    # rerank, ~18 min
mlpl-repl --source-dir . -f demos/atlas/intent.mlpl -- tmp/atlas/intent.json  # intent + Nouls, seconds
```

`just atlas` runs the extraction and both models in order, when a checkout is
where it expects one.
