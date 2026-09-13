#!/usr/bin/env python3
"""Turns the bench JSON reports into one table and one CSV."""
import json
import pathlib
import sys

GIB = 2**30
COLUMNS = [
    ("model", "model", "{}"),
    ("item", "item", "{}"),
    ("prompt", "prompt", "{}"),
    ("modality", "modality", "{}"),
    ("dtype", "dtype", "{}"),
    ("weight_gib", "weight_bytes", "{:.1f}"),
    ("peak_rss_gib", "peak_rss_bytes", "{:.1f}"),
    ("final_rss_gib", "final_rss_bytes", "{:.1f}"),
    ("load_s", "load_secs", "{:.1f}"),
    ("prefill_tok", "prefill_tokens", "{}"),
    ("img_tok", "image_tokens", "{}"),
    ("prefill_s", "prefill_secs", "{:.1f}"),
    ("prefill_tok_s", None, "{:.1f}"),
    ("out_tok", "decode_tokens", "{}"),
    ("decode_tok_s", "decode_tokens_per_sec", "{:.2f}"),
    ("gen_s", "generate_secs", "{:.1f}"),
    ("json", "parses_as_json", "{}"),
    ("capped", "hit_token_cap", "{}"),
]


def rows(root):
    for path in sorted(pathlib.Path(root).rglob("*.json")):
        d = json.load(open(path))
        d["modality"] = "image" if d["with_image"] else "text"
        d["weight_bytes"] /= GIB
        d["peak_rss_bytes"] /= GIB
        d["final_rss_bytes"] /= GIB
        yield d


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "bench-out"
    table = []
    for d in rows(root):
        cells = []
        for name, key, fmt in COLUMNS:
            if name == "prefill_tok_s":
                secs = d["prefill_secs"]
                value = d["prefill_tokens"] / secs if secs else 0.0
            else:
                value = d[key]
            cells.append(fmt.format(value))
        table.append(cells)

    headers = [c[0] for c in COLUMNS]
    widths = [
        max(len(h), *(len(r[i]) for r in table)) if table else len(h)
        for i, h in enumerate(headers)
    ]
    line = "  ".join(h.ljust(w) for h, w in zip(headers, widths))
    print(line)
    print("-" * len(line))
    for r in table:
        print("  ".join(c.ljust(w) for c, w in zip(r, widths)))

    csv_path = pathlib.Path(root) / "summary.csv"
    with open(csv_path, "w") as fh:
        fh.write(",".join(headers) + "\n")
        for r in table:
            fh.write(",".join(r) + "\n")
    print(f"\nwrote {csv_path}")


if __name__ == "__main__":
    main()
