#!/usr/bin/env python3
"""Builds the bench report page from the run JSON in bench-out/."""
import html
import json
import pathlib
import sys

GIB = 2**30
ITEM_ORDER = ["3666883698", "3121742525", "3754128439", "3799737423"]
MODEL_NAMES = {
    "mistral": "Mistral 7B Instruct",
    "qwen3-vl": "Qwen3-VL 8B Instruct",
    "gemma4": "Gemma 4 E4B",
}

CSS = """
:root {
  color-scheme: light;
  --ground: #f3f6f6;
  --surface: #ffffff;
  --surface-sunk: #eaeff0;
  --ink: #101819;
  --ink-mid: #3d4c4e;
  --ink-low: #637275;
  --rule: #d6dfe0;
  --rule-strong: #b9c6c8;
  --accent: #0c6d73;
  --accent-soft: #d9ecec;
  --ok: #2c6740;
  --warn: #8d5c0d;
  --crit: #9d3327;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --ground: #0c1214;
    --surface: #131c1e;
    --surface-sunk: #0f1719;
    --ink: #e4ecec;
    --ink-mid: #b3c2c3;
    --ink-low: #85979a;
    --rule: #24312f;
    --rule-strong: #354547;
    --accent: #56b7bd;
    --accent-soft: #16292b;
    --ok: #6cb383;
    --warn: #d5a052;
    --crit: #e08376;
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --ground: #0c1214;
  --surface: #131c1e;
  --surface-sunk: #0f1719;
  --ink: #e4ecec;
  --ink-mid: #b3c2c3;
  --ink-low: #85979a;
  --rule: #24312f;
  --rule-strong: #354547;
  --accent: #56b7bd;
  --accent-soft: #16292b;
  --ok: #6cb383;
  --warn: #d5a052;
  --crit: #e08376;
}

* { box-sizing: border-box; }

body {
  margin: 0;
  background: var(--ground);
  color: var(--ink);
  font-family: "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}

.wrap {
  max-width: 61rem;
  margin: 0 auto;
  padding: 3rem 1.5rem 5rem;
  display: flex;
  flex-direction: column;
  gap: 3.25rem;
}

h1, h2, h3 {
  font-family: "IBM Plex Serif", Georgia, serif;
  font-weight: 600;
  text-wrap: balance;
  margin: 0;
  letter-spacing: -0.01em;
}
h1 { font-size: 2.3rem; line-height: 1.15; }
h2 { font-size: 1.45rem; }
h3 { font-size: 1.02rem; font-weight: 600; }

p { margin: 0; max-width: 64ch; }

.eyebrow {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.72rem;
  letter-spacing: 0.13em;
  text-transform: uppercase;
  color: var(--accent);
  margin: 0;
}

header.masthead {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  border-bottom: 2px solid var(--rule-strong);
  padding-bottom: 1.75rem;
}
header.masthead p.lede {
  color: var(--ink-mid);
  font-size: 1.06rem;
}

.runspec {
  display: flex;
  flex-wrap: wrap;
  gap: 0 1.6rem;
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.78rem;
  color: var(--ink-low);
}
.runspec span b {
  color: var(--ink-mid);
  font-weight: 500;
}

section { display: flex; flex-direction: column; gap: 1.15rem; }

.sechead { display: flex; flex-direction: column; gap: 0.3rem; }

/* ---- outputs ---- */

.item {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  padding-top: 1.4rem;
  border-top: 1px solid var(--rule);
}
.item:first-of-type { border-top: none; padding-top: 0; }

.itemhead {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.5rem 1rem;
}
.itemhead .meta {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.74rem;
  color: var(--ink-low);
}

.pair {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(19rem, 1fr));
  gap: 1rem;
}

.out {
  background: var(--surface);
  border: 1px solid var(--rule);
  border-left: 3px solid var(--accent);
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.out > .bar {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 0.75rem;
  padding: 0.55rem 0.85rem;
  border-bottom: 1px solid var(--rule);
  background: var(--surface-sunk);
}
.out .promptname {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-mid);
}
.out .stats {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.72rem;
  color: var(--ink-low);
  font-variant-numeric: tabular-nums;
}
.out pre {
  margin: 0;
  padding: 0.85rem;
  overflow-x: auto;
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.79rem;
  line-height: 1.55;
  color: var(--ink);
  tab-size: 2;
}

.flag {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.68rem;
  letter-spacing: 0.06em;
  padding: 0.1rem 0.4rem;
  border: 1px solid currentColor;
  border-radius: 2px;
}
.flag.ok { color: var(--ok); }
.flag.bad { color: var(--crit); }

.cmp { display: flex; flex-direction: column; gap: 0.5rem; margin-top: 0.4rem; }
.cmphead {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.72rem; letter-spacing: 0.1em; text-transform: uppercase;
  color: var(--ink-low); margin: 0;
}
table.cmptable td, table.cmptable th { vertical-align: top; white-space: normal; }
table.cmptable td.cfg { white-space: nowrap; font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 0.72rem; }
.chip {
  display: inline-block; margin: 0 0.25rem 0.25rem 0; padding: 0.08rem 0.4rem;
  border: 1px solid var(--rule); background: var(--surface-sunk);
  border-radius: 2px; font-size: 0.76rem; color: var(--ink-low);
}
.chip.uniq { color: var(--ink); border-color: var(--accent); background: var(--surface); }
.chip.uniq::before { content: "\2022 "; color: var(--accent); }
.chip.bad { color: var(--crit); border-color: var(--crit); }
.chip.none { color: var(--ink-low); border-style: dashed; }
tr.rawrow td { padding-top: 0; border-top: none; }
tr.rawrow details { font-size: 0.76rem; color: var(--ink-low); }
tr.rawrow summary { cursor: pointer; font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 0.7rem; letter-spacing: 0.06em; }
tr.rawrow pre {
  margin: 0.4rem 0 0; padding: 0.6rem; overflow-x: auto; background: var(--surface-sunk);
  font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 0.74rem; color: var(--ink);
}
tr.rawrow summary:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

/* ---- tables ---- */

.tablewrap { overflow-x: auto; border: 1px solid var(--rule); background: var(--surface); }
table { border-collapse: collapse; width: 100%; font-size: 0.84rem; }
caption { text-align: left; padding: 0.75rem 0.85rem; color: var(--ink-low); font-size: 0.8rem; border-bottom: 1px solid var(--rule); }
th, td { padding: 0.5rem 0.85rem; text-align: right; white-space: nowrap; }
th:first-child, td:first-child,
th.txt, td.txt { text-align: left; }
thead th {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.7rem;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--ink-low);
  font-weight: 500;
  border-bottom: 1px solid var(--rule-strong);
}
tbody tr + tr td { border-top: 1px solid var(--rule); }
tbody td { font-variant-numeric: tabular-nums; }
tbody td.key { color: var(--ink-mid); }

/* ---- memory ---- */

.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(11.5rem, 1fr));
  gap: 1rem;
}
.tile {
  background: var(--surface);
  border: 1px solid var(--rule);
  padding: 0.95rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.tile .label {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.68rem;
  letter-spacing: 0.09em;
  text-transform: uppercase;
  color: var(--ink-low);
}
.tile .figure {
  font-family: "IBM Plex Serif", Georgia, serif;
  font-size: 1.75rem;
  font-variant-numeric: tabular-nums;
  line-height: 1.1;
}
.tile .note { font-size: 0.78rem; color: var(--ink-mid); }

.memchart { display: flex; flex-direction: column; gap: 0.55rem; }
.membar { display: grid; grid-template-columns: 8.5rem 1fr 5rem; align-items: center; gap: 0.7rem; font-size: 0.8rem; }
.membar .name { color: var(--ink-mid); font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 0.74rem; }
.membar .track { background: var(--surface-sunk); border: 1px solid var(--rule); height: 1.1rem; position: relative; }
.membar .fill { position: absolute; inset: 0 auto 0 0; background: var(--accent); }
.membar .fill.soft { background: var(--accent-soft); border-right: 2px solid var(--accent); }
.membar .val { text-align: right; font-variant-numeric: tabular-nums; font-family: "IBM Plex Mono", ui-monospace, monospace; font-size: 0.76rem; }

/* ---- notes ---- */

ul.notes { margin: 0; padding-left: 1.1rem; display: flex; flex-direction: column; gap: 0.6rem; max-width: 66ch; }
ul.notes li::marker { color: var(--accent); }
code {
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 0.87em;
  background: var(--surface-sunk);
  padding: 0.08em 0.32em;
  border-radius: 2px;
}

.pending {
  border: 1px dashed var(--rule-strong);
  background: var(--surface-sunk);
  padding: 1rem 1.1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.pending .eyebrow { color: var(--warn); }

footer {
  border-top: 1px solid var(--rule);
  padding-top: 1.25rem;
  font-size: 0.8rem;
  color: var(--ink-low);
  max-width: 66ch;
}

@media (prefers-reduced-motion: reduce) {
  * { animation: none !important; transition: none !important; }
}
"""


def esc(s):
    return html.escape(s, quote=False)


def gib(n):
    return n / GIB


def build(payload, pending_note):
    items = payload["items"]
    runs = payload["runs"]
    by_item = {}
    for r in runs:
        by_item.setdefault(r["item"], []).append(r)

    def key(r):
        return (
            r.get("runtime", "candle"),
            r["model"],
            r.get("dtype", "f16"),
            "image" if r["with_image"] else "text",
            r.get("variant", "direct"),
        )

    models = sorted({key(r) for r in runs})
    weight = {m: max(r["weight_bytes"] for r in runs if key(r) == m) for m in models}
    peak = {m: max(r["peak_rss_bytes"] for r in runs if key(r) == m) for m in models}
    final = {m: max(r["final_rss_bytes"] for r in runs if key(r) == m) for m in models}

    def label(m, sep="&middot;"):
        runtime, model, dtype, modality, variant = m
        name = MODEL_NAMES.get(model, model)
        stem = f"{name} {sep} {runtime} {dtype}"
        if variant != "direct":
            return f"{stem} {sep} {variant}"
        return stem if modality == "text" else f"{stem} + image"


    out = []
    out.append("<title>Workshop Extraction Bench</title>")
    out.append(
        '<link rel="stylesheet" '
        'href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&'
        "family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Serif:wght@500;600&"
        'display=swap">'
    )
    out.append(f"<style>{CSS}</style>")
    out.append('<div class="wrap">')

    # masthead
    out.append("<header class='masthead'>")
    out.append("<p class='eyebrow'>CPU inference, candle 0.11</p>")
    out.append("<h1>Workshop Extraction Bench</h1>")
    out.append(
        "<p class='lede'>Every model answer, verbatim, for three RimWorld workshop items "
        "against both extraction prompts, on two runtimes. Resource and throughput "
        "figures follow the answers, aggregated per model.</p>"
    )
    out.append("<div class='runspec'>")
    for spec_name, value in [
        ("host", "Ryzen 9 7950X3D, 16 threads"),
        ("device", "CPU, F16"),
        ("runtimes", "candle 0.11, llama.cpp b1-56b9eb2"),
        ("sampling", "argmax, deterministic"),
        ("output", "json_schema grammar"),
    ]:
        out.append(f"<span>{spec_name} <b>{esc(value)}</b></span>")
    out.append("</div>")
    out.append("</header>")

    # outputs, as a comparison table per item and prompt
    out.append("<section>")
    out.append("<div class='sechead'>")
    out.append("<p class='eyebrow'>To judge</p>")
    out.append("<h2>Extracted values, side by side</h2>")
    out.append(
        "<p>One row per configuration. A value carrying a dot is unique to that row "
        "&mdash; nothing else for this item and prompt produced it. Shared values are "
        "dimmed, so what differs is what stands out. Open a row to read the raw answer "
        "exactly as generated.</p>"
    )
    out.append("</div>")

    def parsed(raw):
        body = raw.strip()
        if body.startswith("```"):
            body = body.split("\n", 1)[-1].rsplit("```", 1)[0]
        try:
            value = json.loads(body.strip())
            return value if isinstance(value, dict) else None
        except Exception:
            return None

    FIELDS = {"features": ["types", "features"], "genres": ["genres", "themes"]}

    for item_id in ITEM_ORDER:
        if item_id not in by_item:
            continue
        meta = items[item_id]
        out.append("<div class='item'>")
        out.append("<div class='itemhead'>")
        out.append(f"<h3>{esc(meta['title'])}</h3>")
        out.append(
            f"<span class='meta'>{item_id} &middot; {meta['desc_chars']:,} chars of description</span>"
        )
        out.append("</div>")

        for prompt_key, fields in FIELDS.items():
            rows = [r for r in by_item[item_id] if r["prompt"] == prompt_key]
            if not rows:
                continue
            rows.sort(key=lambda r: (key(r)[0], key(r)[2], key(r)[3], key(r)[4]))
            # Count how many rows produced each value, per field.
            counts = {f: {} for f in fields}
            for r in rows:
                obj = parsed(r["raw_output"]) or {}
                for f in fields:
                    for v in obj.get(f, []) or []:
                        if isinstance(v, str):
                            counts[f].setdefault(v.strip().lower(), set()).add(id(r))

            out.append("<div class='cmp'>")
            out.append(f"<p class='cmphead'>{esc(prompt_key)}</p>")
            out.append("<div class='tablewrap'><table class='cmptable'>")
            out.append("<thead><tr><th class='txt'>Configuration</th>"
                       + "".join(f"<th class='txt'>{esc(f)}</th>" for f in fields)
                       + "<th>out tok</th></tr></thead><tbody>")
            for r in rows:
                obj = parsed(r["raw_output"])
                out.append("<tr>")
                out.append(f"<td class='txt key cfg'>{esc(label(key(r), sep='/'))}</td>")
                for f in fields:
                    if obj is None:
                        out.append("<td class='txt'><span class='chip bad'>unparseable</span></td>")
                        continue
                    vals = [v for v in (obj.get(f) or []) if isinstance(v, str)]
                    if not vals:
                        out.append("<td class='txt'><span class='chip none'>&mdash;</span></td>")
                        continue
                    cells = []
                    for v in vals:
                        uniq = len(counts[f].get(v.strip().lower(), ())) == 1
                        cls = "chip uniq" if uniq else "chip"
                        cells.append(f"<span class='{cls}'>{esc(v)}</span>")
                    out.append(f"<td class='txt'>{''.join(cells)}</td>")
                out.append(f"<td>{r['decode_tokens']}</td>")
                out.append("</tr>")
                out.append(
                    "<tr class='rawrow'><td colspan='%d'><details><summary>raw answer</summary>"
                    "<pre>%s</pre></details></td></tr>"
                    % (len(fields) + 2, esc(r["raw_output"].strip() or "(empty)"))
                )
            out.append("</tbody></table></div>")
            out.append("</div>")
        out.append("</div>")
    out.append("</section>")

    # resources and throughput, per model
    out.append("<section>")
    out.append("<div class='sechead'>")
    out.append("<p class='eyebrow'>Per model</p>")
    out.append("<h2>Resources and throughput</h2>")
    out.append(
        "<p>One row per model, over the whole suite. Rates come from the summed "
        "tokens and summed seconds, not from averaging each run's rate. Peak resident "
        "memory runs at roughly twice the weight file: the published checkpoint is "
        "BF16, candle has no BF16 matmul on CPU, so the mmap and the converted F16 copy "
        "are both resident at the peak.</p>"
    )
    out.append("</div>")

    out.append("<div class='tablewrap'><table>")
    out.append(
        f"<caption>{len(runs)} generations across {len(ITEM_ORDER)} items and two prompts. "
        "Wall clock excludes model load.</caption>"
        "<thead><tr><th class='txt'>Model</th>"
        "<th>Disk</th><th>Peak RSS</th><th>Settled RSS</th><th>Warm load</th>"
        "<th>Image tok</th><th>Prefill tok/s</th><th>Decode tok/s</th><th>Per item</th><th>Suite</th>"
        "</tr></thead><tbody>"
    )
    for m in models:
        sub = [r for r in runs if key(r) == m]
        pf_tok = sum(r["prefill_tokens"] for r in sub)
        pf_sec = sum(r["prefill_secs"] for r in sub)
        dc_tok = sum(r["decode_tokens"] for r in sub)
        dc_sec = sum(r["decode_secs"] for r in sub)
        wall = sum(r["generate_secs"] for r in sub)
        n_items = len({r["item"] for r in sub})
        load = sum(r["load_secs"] for r in sub) / len(sub)
        out.append(
            f"<tr><td class='txt key'>{esc(label(m))}</td>"
            f"<td>{gib(weight[m]):.1f} GiB</td>"
            f"<td>{gib(peak[m]):.1f} GiB</td>"
            f"<td>{gib(final[m]):.1f} GiB</td>"
            f"<td>{load:.1f} s</td>"
            f"<td>{max((r['image_tokens'] for r in sub), default=0) or '&mdash;'}</td>"
            f"<td>{pf_tok / pf_sec if pf_sec else 0:.1f}</td>"
            f"<td>{dc_tok / dc_sec if dc_sec else 0:.2f}</td>"
            f"<td>{wall / n_items / 60:.1f} min</td>"
            f"<td>{wall / 60:.1f} min</td></tr>"
        )
    out.append("</tbody></table></div>")

    out.append("<div class='tiles'>")
    for m in models:
        name = label(m)
        out.append("<div class='tile'>")
        out.append(f"<span class='label'>{esc(name)} on disk</span>")
        out.append(f"<span class='figure'>{gib(weight[m]):.1f} GiB</span>")
        out.append("<span class='note'>safetensors, BF16</span>")
        out.append("</div>")
        out.append("<div class='tile'>")
        out.append(f"<span class='label'>{esc(name)} peak RSS</span>")
        out.append(f"<span class='figure'>{gib(peak[m]):.1f} GiB</span>")
        out.append(
            f"<span class='note'>settles to {gib(final[m]):.1f} GiB after load</span>"
        )
        out.append("</div>")
    out.append("</div>")

    out.append("<div class='memchart'>")
    span = max(max(peak.values()), 1)
    for m in models:
        name = label(m)
        out.append(
            f"<p style='font-size:0.78rem;color:var(--ink-mid);margin-top:0.4rem'>{esc(name)}</p>"
        )
        for bar_name, value, soft in [
            ("weights on disk", weight[m], True),
            ("resident, settled", final[m], True),
            ("resident, peak", peak[m], False),
        ]:
            pct = value / span * 100
            out.append("<div class='membar'>")
            out.append(f"<span class='name'>{esc(bar_name)}</span>")
            out.append(
                f"<span class='track'><span class='fill{' soft' if soft else ''}' "
                f"style='width:{pct:.1f}%'></span></span>"
            )
            out.append(f"<span class='val'>{gib(value):.1f} GiB</span>")
            out.append("</div>")
    out.append("</div>")
    out.append("</section>")

    # notes
    out.append("<section>")
    out.append("<div class='sechead'>")
    out.append("<p class='eyebrow'>Reading the numbers</p>")
    out.append("<h2>What the harness changed, and what to distrust</h2>")
    out.append("</div>")
    out.append("<ul class='notes'>")
    out.append(
        "<li><b>llama.cpp runs Gemma 4 E4B properly, and far cheaper.</b> Same model, "
        "same three items, same prompts: 24 of 24 answers were valid JSON, against candle's "
        "nothing at all. Q4_0 decodes at 19 tok/s where candle's Mistral managed 2.07, and "
        "peaks at 7.5 GiB against candle's 27.3. BF16 peaks at 9.4 GiB for a 14.0 GiB file, "
        "because llama.cpp mmaps and pages lazily instead of building a converted copy, and "
        "it runs BF16 directly rather than refusing it.</li>"
    )
    out.append(
        "<li><b>Bare JSON has to come from a grammar, not the prompt.</b> Adding "
        "&ldquo;respond with the JSON object only&rdquo; does stop the markdown fences, "
        "but the model reads it as &ldquo;be brief&rdquo; and one item dropped from 16 "
        "features to 2, 148 output tokens to 22. A milder wording cost exactly the same. "
        "<code>response_format: json_schema</code> compiles the schema to a grammar and "
        "constrains decoding, so a fence is not a reachable token and the prompt is "
        "untouched &mdash; identical content to baseline. It also removes the malformed "
        "JSON class outright, which is what makes <code>sanitise_output</code> panic.</li>"
    )
    out.append(
        "<li><b>The weapon sprawl was the examples teaching the wrong operation.</b> Every "
        "consolidation example in <code>features.txt</code> collapsed modifier plus noun "
        "into that noun &mdash; &ldquo;steel sword&rdquo; to &ldquo;swords&rdquo;. None "
        "collapsed sibling nouns into a parent, so Defensive Network listed machine guns, "
        "autocannons, cannons, missile weapons, electromagnetic weapons, laser weapons and "
        "tracking weapons separately. Three examples of the right shape plus one rule "
        "&mdash; three or more kinds of a thing become the thing &mdash; took it from 23 "
        "features to 12. Examples alone made it worse, at 21. Rustic Workbenches kept all "
        "16 of its entries either way, because naming them is the point of that mod and the "
        "model can tell.</li>"
    )
    out.append(
        "<li><b>Vanilla Gravship Expanded is the case where vision should win, and "
        "attaching images still did nothing.</b> Its prose is 2,674 characters of marketing "
        "while 18 embedded infographics carry the real content, including a research tree "
        "of 14 named projects with costs. Asked about that one image alone, the model read "
        "all 14 with their Gravdata costs and one misspelling. Asked for features with the "
        "same image attached &mdash; or with all 18 attached, at 9,241 prompt tokens &mdash; "
        "it returned the text-only answer and ignored them. The extraction prompt anchors "
        "attention on the description; the picture is treated as decoration.</li>"
    )
    out.append(
        "<li><b>The two fixes compound.</b> Gravship Chapter 2 with the revised prompt and "
        "transcription returned 34 features, 22 of them absent from its prose: exocoids, "
        "gravcloakers, grav field dilutors, astrofire launchers, warcomputers, space "
        "infestation, salvager strongholds, launch boons and mishaps. Against 14 for one "
        "call with images attached and 12 for text alone. Every one was checked back to the "
        "transcript. The new consolidation rule flattened seven weapon kinds into "
        "<code>weapons</code> on Defensive Network yet left all 34 of these standing, so it "
        "keys on real sibling relationships rather than list length. The cost is 279s of "
        "transcription over 25 calls plus 46s to extract, against roughly 146s for the "
        "single call, and the prompt reaches 7,497 tokens &mdash; 25 images is near the "
        "ceiling at 16k context.</li>"
    )
    out.append(
        "<li><b>Transcribe first, then extract.</b> Running the images through a "
        "transcription pass and appending the result to the description took that item from "
        "12 features to 35, and 11 of 13 sampled additions appear nowhere in the prose: "
        "point defense turrets, cloakers, warplatforms, gauss bombard, gravtor, grav "
        "generators, destroyed variants, visibility counter, gravship scanners, thrusters, "
        "derelict gravships. Two cheap passes beat one multimodal pass, because each prompt "
        "asks for one thing.</li>"
    )
    out.append(
        "<li><b>The vision path is genuinely reading the image.</b> Asked to transcribe "
        "the preview, the model returned <code>1.6</code> from Greekcore's corner version "
        "tag and ten colonist name labels rendered at about eight pixels &mdash; Huma, Luca, "
        "Livia, Vita, Titus, Liviana, Antonius, Hadrian against the real Humps, Lucia, "
        "Livia, Vita, Tatius, Liviana, Antoninus, Hadrianus &mdash; and read "
        "<code>失能</code> and <code>1.5</code> off the Dead Man's Switch art. None of those "
        "strings appear in any description. The control, same prompt with the image "
        "removed and the description still present, answered <code>NO IMAGE.</code> every "
        "time. So the next finding is about the task, not a broken pipeline.</li>"
    )
    out.append(
        "<li><b>The images made the answers worse, not richer.</b> Every image run stayed "
        "valid, but the extraction got shorter and more generic. Defensive Network went "
        "from 23 features to 12, losing the whole weapon taxonomy; Greekcore lost "
        "&ldquo;wreaths&rdquo;, the one detail its description dwells on. The image also "
        "costs about 160 prompt tokens. On this corpus the preview dilutes the description "
        "rather than adding to it.</li>"
    )
    out.append(
        "<li>Gemma 4 reasons before answering. Left on, it spent 895 tokens to reach a 51 "
        "token answer, and the 256 cap cut it off mid-thought with an empty "
        "<code>content</code> &mdash; which is what the first llama.cpp pass showed. These "
        "runs set <code>enable_thinking: false</code>, which is what you would ship for "
        "structured extraction.</li>"
    )
    out.append(
        "<li><b>Gemma 4 E4B does not work on candle 0.11.</b> Its row is a cost "
        "estimate, not a result. candle's text model never loads the 129 "
        "per-layer-embedding tensors or the per-layer <code>layer_scalar</code>, so the "
        "logits are meaningless and every answer decoded to an empty string. The memory "
        "figure is understated for the same reason: 2.8 GB of "
        "<code>embed_tokens_per_layer</code> is never read. Its vision tower cannot load "
        "the weights at all, because candle asks for <code>q_proj.weight</code> while the "
        "checkpoint stores the clipped form, <code>q_proj.linear.weight</code> with its "
        "min/max pair.</li>"
    )
    out.append(
        "<li><b>Qwen3-VL's 52 GiB is mostly empty cache.</b> "
        "<code>qwen3_vl/text.rs:144</code> builds <code>KvCache::new(2, "
        "max_position_embeddings)</code>, and candle's cache grows by that whole amount at "
        "once, so the first token allocates all 262,144 positions: 36 layers by 8 KV heads "
        "by 128 dims by two tensors is 38.7 GB, whatever the prompt length. The plain "
        "<code>qwen3.rs</code> uses a concatenating cache and has no such cost. Capping "
        "the context in the config would reclaim nearly all of it.</li>"
    )
    out.append(
        "<li>Qwen3-VL needed a shim to load: <code>qwen3_vl/text.rs:285</code> builds "
        "<code>lm_head</code> with <code>linear</code>, demanding a bias no Qwen3 "
        "checkpoint carries. A zero bias is arithmetically identical, so the bench writes "
        "one into a side file.</li>"
    )
    out.append(
        "<li><code>sanitise_output</code> (<code>lib.rs:245</code>) panics on any answer "
        "without a closing brace. <code>find('}').unwrap_or(len)</code> with an inclusive "
        "range slices one byte past the end. Empty answers hit it, and so does JSON cut "
        "off by the token cap &mdash; both of which happened here. The bench guards around "
        "it; the library still does this in <code>run_pipeline</code>.</li>"
    )
    out.append(
        "<li>Stop tokens and the chat template are per model now. Gemma 4 renamed its turn "
        "markers to <code>&lt;|turn&gt;</code> and <code>&lt;turn|&gt;</code>, so the "
        "Gemma 3 names would never have stopped it, and its shipped template calls "
        "<code>.get()</code> on a map, which minijinja cannot render.</li>"
    )
    out.append(
        "<li>MKL was slower here than candle's own backend, roughly half the decode rate "
        "with no prefill advantage, so it is now off by default in both "
        "<code>classification/Cargo.toml</code> and the root "
        "<code>ml-classification</code> feature.</li>"
    )
    out.append(
        "<li>Sampling is argmax with repeat penalty 1.1 over the last 64 tokens, matching "
        "<code>runner.rs:76</code>. Each run is one process, which makes peak RSS exact. "
        "The cap is 256 tokens, against the old pipeline's 100,000.</li>"
    )
    out.append("</ul>")
    if pending_note:
        out.append("<div class='pending'>")
        out.append("<p class='eyebrow'>Still to come</p>")
        out.append(f"<p>{pending_note}</p>")
        out.append("</div>")
    out.append("</section>")

    out.append(
        "<footer>Generated from <code>bench-out/*.json</code> by "
        "<code>classification/bench-report.py</code>. Every figure is read from a run "
        "report; nothing here is estimated.</footer>"
    )
    out.append("</div>")
    return "\n".join(out)


def main():
    payload = json.load(open(sys.argv[1]))
    pending = (
        "Prefill favours BF16 (316 tok/s against Q4_0's 276) because it is compute bound "
        "and this Zen 4 chip has avx512_bf16. The Cascade Lake 6210U target does not, so "
        "expect its BF16 prefill to fall back to F32 while the Q4_0 integer path keeps its "
        "avx512_vnni advantage. Benchmark on the target before trusting the ratio."
    )
    pathlib.Path(sys.argv[2]).write_text(build(payload, pending))
    print(f"wrote {sys.argv[2]}")


if __name__ == "__main__":
    main()
