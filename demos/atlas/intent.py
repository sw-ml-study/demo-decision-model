#!/usr/bin/env python3
"""Featurize sw-atlas's question sets for the intent Choice and two Nouls.

The rerank experiment asks whether the model can pick a resource. This asks the
other half of sw-atlas's design, the half its own history says a matcher is bad
at: what the visitor *wants* (intent), and two yes-or-no questions the program
needs before it answers at all -- is this a question about the docent itself,
and is it about something the catalog does not cover.

All six frozen question sets are used, split by question with a stable hash, so
the class balance of each split follows the sets themselves.

usage: intent.py [--benchmark FILE] [--out FILE]
"""

import argparse
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
WIDTH = 16
MAX_VOCAB = 3000
# The intents worth separating; everything rarer is folded into the commonest,
# because a class with four examples measures nothing.
INTENTS = ["FindResource", "Navigate", "Explain", "Meta", "Unsupported"]


def words(s):
    return re.findall(r"[a-z0-9]+", s.lower())


def tokens(s):
    ws = words(s)
    out = ws[:WIDTH]
    for a, b in zip(ws, ws[1:]):
        if len(out) >= WIDTH:
            break
        out.append(f"{a}_{b}")
    return out


def stable(text):
    h = 7
    for b in text.encode():
        h = (h * 31 + b) % 2147483647
    return h


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--benchmark", default=str(ROOT / "tmp/atlas/benchmark.json"))
    ap.add_argument("--out", default=str(ROOT / "tmp/atlas/intent.json"))
    args = ap.parse_args()
    b = json.loads(Path(args.benchmark).read_text())
    rows = [q for rows in b["questions"].values() for q in rows]
    for q in rows:
        q["label"] = INTENTS.index(q["intent"]) if q["intent"] in INTENTS else 0
        q["meta"] = 1 if q["set"] == "meta" else 0
        q["off"] = 1 if q["set"] == "off-topic" else 0
    # Stratified by set: a hash over all 386 put one meta question and two
    # off-topic ones in the test split, which measures nothing. Every fifth
    # question *of each set* is held out instead, so the rare sets are
    # represented in proportion.
    train, test = [], []
    for name in sorted({q["set"] for q in rows}):
        of_set = [q for q in rows if q["set"] == name]
        of_set.sort(key=lambda q: stable(q["text"]))
        for i, q in enumerate(of_set):
            (test if i % 5 == 4 else train).append(q)

    count = Counter()
    for q in train:
        count.update(tokens(q["text"]))
    vocab = [t for t, n in sorted(count.items(), key=lambda x: (-x[1], x[0])) if n >= 2][:MAX_VOCAB]
    vid = {t: i + 1 for i, t in enumerate(vocab)}

    def batch(qs):
        ids, mask = [], []
        for q in qs:
            toks = tokens(q["text"])
            for i in range(WIDTH):
                r = vid.get(toks[i]) if i < len(toks) else None
                ids.append(r or 0)
                mask.append(1 if r else 0)
        return {
            "n": len(qs),
            "ids": ids,
            "mask": mask,
            "intent": [q["label"] for q in qs],
            "nouls": [v for q in qs for v in (q["meta"], q["off"])],
        }

    data = {"width": WIDTH, "vocab": vocab, "intents": INTENTS,
            "train": batch(train), "test": batch(test)}
    Path(args.out).write_text(json.dumps(data))
    dist = Counter(INTENTS[q["label"]] for q in test)
    majority = max(dist.values()) / len(test)
    print(f"{len(rows)} questions from six frozen sets: {len(train)} train, {len(test)} test")
    print(f"test intents: {dict(dist)}")
    print(f"majority-class baseline on the test split: {majority:.3f}")
    print(f"meta {sum(q['meta'] for q in test)} and off-topic {sum(q['off'] for q in test)} in test")
    print(f"vocabulary {len(vocab)}; wrote {args.out}")


if __name__ == "__main__":
    main()
