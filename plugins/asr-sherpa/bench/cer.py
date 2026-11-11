"""Rough CER: python3 -I cer.py <ref.txt> <hyp.vtt>

Both sides: drop punctuation and whitespace, lowercase. Arabic numerals in the
hypothesis (from ITN) are also rewritten as Chinese numerals for a second figure.
Distance is character-level Levenshtein over the whole concatenated text.
"""
import re
import sys
import unicodedata

DIG = "零一二三四五六七八九"


def zh_num(n: int) -> str:
    if n < 10:
        return DIG[n]
    out = ""
    for value, unit in ((1000, "千"), (100, "百"), (10, "十")):
        q, n = divmod(n, value)
        if q:
            out += ("" if (value == 10 and q == 1 and not out) else DIG[q]) + unit
        elif out and n:
            out += "零" if not out.endswith("零") else ""
    if n:
        out += DIG[n]
    return out


def norm(s: str, numerals: bool) -> str:
    if numerals:
        s = re.sub(r"\d+", lambda m: zh_num(int(m.group())) if len(m.group()) < 5 else m.group(), s)
    return "".join(
        c.lower() for c in s if not (c.isspace() or unicodedata.category(c).startswith("P"))
    )


def lev(a: str, b: str) -> int:
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def vtt_text(path: str) -> str:
    lines = []
    for line in open(path, encoding="utf-8").read().split("\n"):
        if not line or line.startswith(("WEBVTT", "NOTE")) or "-->" in line:
            continue
        lines.append(line)
    return "".join(lines)


ref_raw = open(sys.argv[1], encoding="utf-8").read()
hyp_raw = vtt_text(sys.argv[2])
for numerals in (False, True):
    ref, hyp = norm(ref_raw, False), norm(hyp_raw, numerals)
    d = lev(ref, hyp)
    label = "numerals folded" if numerals else "as written"
    print(f"{label}: ref={len(ref)} hyp={len(hyp)} edits={d} CER={d / len(ref):.2%}")
