#!/usr/bin/env python3
"""Read sw-atlas's corpus and frozen question sets into a benchmark.

sw-atlas asked whether a PR05 card scorer can pick a resource whose card was
never in training -- the row in its own design table that reads "a new post
published last night ... the rerank head scores cards, not labels, so it can
pick it without retraining". `DC01` measured that failing badly on this
repository's own data, where a card is a 1966 decomposition rule and shares
nothing with the input but function words. sw-atlas's regime is different, so
the question has to be asked again, there.

Nothing of sw-atlas's is written or copied here: this reads its checkout and
emits only the featurized arrays the MLPL trainer needs, into tmp/. Point it at
a checkout with `--atlas`.

usage: extract.py [--atlas DIR] [--out FILE] [--held-out N] [--width W]
"""

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_ATLAS = ROOT.parent.parent / "software-wrighter-lab/sw-atlas"

RESOURCE = re.compile(
    r'Resource\(\s*id:\s*ResourceId\("([^"]+)"\).*?'
    r'kind:\s*(\w+).*?'
    r'title:\s*"((?:[^"\\]|\\.)*)".*?'
    r'summary:\s*"((?:[^"\\]|\\.)*)"',
    re.S,
)
CONCEPTS = re.compile(r"concepts:\s*\[(.*?)\]", re.S)
ALIASES = re.compile(r"aliases:\s*\[(.*?)\]", re.S)
QUESTION = re.compile(
    r'\(\s*text:\s*"((?:[^"\\]|\\.)*)"\s*,\s*expect:\s*\[([^\]]*)\]\s*,\s*intent:\s*(\w+)', re.S
)


def unescape(s):
    return s.replace('\\"', '"').replace("\\n", " ").replace("\\\\", "\\")


def resources(atlas: Path):
    """Every resource as a card: what a visitor could match it by."""
    text = (atlas / "build/corpus/corpus.ron").read_text()
    out = {}
    # Split on the record boundary first so the non-greedy fields cannot run
    # from one resource into the next.
    for chunk in text.split("Resource(")[1:]:
        m = RESOURCE.search("Resource(" + chunk)
        if not m:
            continue
        rid, kind, title, summary = m.group(1), m.group(2), unescape(m.group(3)), unescape(m.group(4))
        concepts = re.findall(r'ConceptId\("([^"]+)"\)', CONCEPTS.search(chunk).group(1)) if CONCEPTS.search(chunk) else []
        aliases = re.findall(r'"([^"]*)"', ALIASES.search(chunk).group(1)) if ALIASES.search(chunk) else []
        out[rid] = {
            "id": rid,
            "kind": kind,
            "title": title,
            "summary": summary,
            "concepts": [c.replace("-", " ") for c in concepts],
            "aliases": aliases,
        }
    return out


def questions(atlas: Path, name: str):
    rows = []
    path = atlas / f"sources/questions/{name}.ron"
    if not path.exists():
        return rows
    for m in QUESTION.finditer(path.read_text()):
        expect = re.findall(r'"([^"]+)"', m.group(2))
        rows.append({"text": unescape(m.group(1)), "expect": expect, "intent": m.group(3), "set": name})
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--atlas", default=str(DEFAULT_ATLAS))
    ap.add_argument("--out", default=str(ROOT / "tmp/atlas/benchmark.json"))
    args = ap.parse_args()
    atlas = Path(args.atlas)
    if not (atlas / "build/corpus/corpus.ron").exists():
        raise SystemExit(f"no built corpus under {atlas}; run its `just corpus` there first")

    cards = resources(atlas)
    sets = {n: questions(atlas, n) for n in
            ("paraphrase", "exact-name", "ambiguous", "follow-up", "meta", "off-topic")}
    single = [q for q in sets["paraphrase"] if len(q["expect"]) == 1 and q["expect"][0] in cards]
    print(f"corpus: {len(cards)} resources")
    kinds = {}
    for c in cards.values():
        kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
    print("  by kind:", ", ".join(f"{k} {v}" for k, v in sorted(kinds.items(), key=lambda x: -x[1])))
    for n, rows in sets.items():
        print(f"  {n:<12} {len(rows):>4} questions")
    print(f"\nparaphrases naming exactly one resource that exists: {len(single)}")
    answered = {q["expect"][0] for q in single}
    print(f"distinct resources they point at: {len(answered)} of {len(cards)}")
    missing = [q for q in sets["paraphrase"] if len(q["expect"]) == 1 and q["expect"][0] not in cards]
    if missing:
        print(f"  ({len(missing)} paraphrases name an id not in the corpus, e.g. {missing[0]['expect'][0]})")
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({"cards": list(cards.values()), "questions": sets}, indent=1))
    print(f"\nwrote {out}")


if __name__ == "__main__":
    main()
