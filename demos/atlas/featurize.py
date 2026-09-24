#!/usr/bin/env python3
"""Featurize the sw-atlas benchmark for the MLPL card scorer.

Splits by *resource*, not by question: a held-out resource's card is never
offered during training and none of its questions are trained on, so the
questions that point at it are the "a post published last night" case sw-atlas
needs an answer about. Everything else is the warm case.

Emits the arrays `demos/atlas/train.mlpl` reads. Nothing of sw-atlas's is
written into this repository; the benchmark JSON lives in tmp/.

usage: featurize.py [--benchmark FILE] [--out FILE] [--hold-out FRACTION]
"""

import argparse
import json
import math
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
QUERY_WIDTH = 16
CARD_WIDTH = 64
MAX_VOCAB = 6000
NEGATIVES = 15


def words(s):
    return re.findall(r"[a-z0-9]+", s.lower())


def tokens(s, width):
    """Words then adjacent bigrams, as the MLPL featurizer does."""
    ws = words(s)
    out = ws[:width]
    for a, b in zip(ws, ws[1:]):
        if len(out) >= width:
            break
        out.append(f"{a}_{b}")
    return out


def card_text(c):
    return " ".join([c["title"], " ".join(c["aliases"]), " ".join(c["concepts"]), c["summary"]])


def stable_split(ids, fraction):
    """Hold out a fraction of resources by a stable hash of the id."""
    held = set()
    for rid in ids:
        h = 7
        for b in rid.encode():
            h = (h * 31 + b) % 2147483647
        if h % 100 < fraction * 100:
            held.add(rid)
    return held


def matcher_ranking(query, docs, idf):
    """The deterministic first stage: IDF-weighted token overlap."""
    qs = set(words(query))
    scored = [(sum(idf.get(w, 0.0) for w in qs if w in docs[i]), i) for i in docs]
    scored.sort(key=lambda x: (-x[0], x[1]))
    return [i for _, i in scored]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--benchmark", default=str(ROOT / "tmp/atlas/benchmark.json"))
    ap.add_argument("--out", default=str(ROOT / "tmp/atlas/data.json"))
    ap.add_argument("--hold-out", type=float, default=0.25)
    args = ap.parse_args()

    b = json.loads(Path(args.benchmark).read_text())
    cards = b["cards"]
    index = {c["id"]: i for i, c in enumerate(cards)}
    queries = [q for q in b["questions"]["paraphrase"] if len(q["expect"]) == 1 and q["expect"][0] in index]

    answered = sorted({q["expect"][0] for q in queries})
    held = stable_split(answered, args.hold_out)
    warm_q = [q for q in queries if q["expect"][0] not in held]
    cold_q = [q for q in queries if q["expect"][0] in held]
    # Of the warm questions, a fifth are kept back to measure the ordinary case.
    train_q = [q for i, q in enumerate(warm_q) if i % 5 != 4]
    warm_test = [q for i, q in enumerate(warm_q) if i % 5 == 4]

    # Vocabulary: the cards the model may train on, plus the training queries.
    count = Counter()
    for c in cards:
        if c["id"] not in held:
            count.update(tokens(card_text(c), CARD_WIDTH))
    for q in train_q:
        count.update(tokens(q["text"], QUERY_WIDTH))
    vocab = [t for t, n in sorted(count.items(), key=lambda x: (-x[1], x[0])) if n >= 2][:MAX_VOCAB]
    vid = {t: i + 1 for i, t in enumerate(vocab)}

    def matrix(texts, width):
        ids, mask = [], []
        for t in texts:
            toks = tokens(t, width)
            for i in range(width):
                r = vid.get(toks[i]) if i < len(toks) else None
                ids.append(r or 0)
                mask.append(1 if r else 0)
        return ids, mask

    # The matcher baseline, and the candidate lists the hybrid would rerank.
    docs = {c["id"]: Counter(words(card_text(c))) for c in cards}
    n = len(docs)
    df = Counter()
    for t in docs.values():
        df.update(set(t))
    idf = {w: math.log(n / (1 + k)) for w, k in df.items()}

    def rankings(qs):
        out = []
        for q in qs:
            order = matcher_ranking(q["text"], docs, idf)
            out.append([index[i] for i in order[:50]])
        return out

    card_ids, card_mask = matrix([card_text(c) for c in cards], CARD_WIDTH)
    # The scorer's question axis: one question, asked of every row. sw-atlas
    # would put intent here; this benchmark does not need it yet.
    q_ids, q_mask = matrix(["which resource is this asking for"], QUERY_WIDTH)
    sets = {}
    for name, qs in (("train", train_q), ("warm", warm_test), ("cold", cold_q)):
        qi, qm = matrix([q["text"] for q in qs], QUERY_WIDTH)
        sets[name] = {
            "n": len(qs),
            "ids": qi,
            "mask": qm,
            "gold": [index[q["expect"][0]] for q in qs],
            "top50": [c for r in rankings(qs) for c in r],
            "texts": [q["text"] for q in qs],
        }

    # Training candidates: the gold card plus NEGATIVES others, all warm, drawn
    # by a stable walk so the run is reproducible.
    trainable = [i for i, c in enumerate(cards) if c["id"] not in held]
    cand = []
    for k, q in enumerate(train_q):
        gold = index[q["expect"][0]]
        chosen = [gold]
        h = 7 + k
        while len(chosen) < NEGATIVES + 1:
            h = (h * 1103515245 + 12345) % 2147483647
            pick = trainable[h % len(trainable)]
            if pick not in chosen:
                chosen.append(pick)
        cand.append(sorted(chosen))
    data = {
        "question": {"ids": q_ids, "mask": q_mask},
        "query_width": QUERY_WIDTH,
        "card_width": CARD_WIDTH,
        "vocab": vocab,
        "cards": {"n": len(cards), "ids": card_ids, "mask": card_mask,
                  "held_out": [i for i, c in enumerate(cards) if c["id"] in held],
                  "titles": [c["title"] for c in cards]},
        "train": {**sets["train"], "candidates": [c for row in cand for c in row],
                  "per_query": NEGATIVES + 1},
        "warm": sets["warm"],
        "cold": sets["cold"],
    }
    Path(args.out).write_text(json.dumps(data))
    print(f"cards {len(cards)} ({len(held)} resources held out entirely, their cards never offered in training)")
    print(f"queries: {len(train_q)} train, {len(warm_test)} warm test, {len(cold_q)} cold test")
    print(f"vocabulary {len(vocab)}; query width {QUERY_WIDTH}, card width {CARD_WIDTH}")
    for name in ("warm", "cold"):
        s = sets[name]
        hits = [1 for g, row in zip(s["gold"], [s["top50"][i * 50:(i + 1) * 50] for i in range(s["n"])]) if g in row[:20]]
        print(f"  matcher recall@20 on {name}: {len(hits) / max(1, s['n']):.3f} ({s['n']} questions)")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
