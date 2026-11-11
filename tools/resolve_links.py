"""Check that relative Markdown links in a repo resolve: the file exists and the #anchor is an explicit <a id> or a heading slug.
Usage: python3 -I resolve_links.py [repo_root]  (defaults to cwd). Prints broken links and 'N broken'."""
import os, re, subprocess, sys, unicodedata

root = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
files = subprocess.run(["git", "ls-files", "*.md"], cwd=root, capture_output=True, text=True, check=True, timeout=30).stdout.split("\n")
files = [f for f in files if f]

def slug(h):
    h = re.sub(r"`|\*|\[|\]|\(.*?\)", "", h).strip().lower()
    out = []
    for ch in h:
        cat = unicodedata.category(ch)
        if ch in " -":
            out.append("-")
        elif ch == "_" or cat[0] in "LN" or cat == "Mn":
            out.append(ch)
    return "".join(out)

anchors = {}
texts = {}
for f in files:
    s = open(os.path.join(root, f), encoding="utf-8").read()
    s_nocode = re.sub(r"(?ms)^```.*?^```", "", s)
    ids = set(re.findall(r'<a id="([^"]+)"', s))
    for m in re.finditer(r"(?m)^#{1,6}\s+(.+?)\s*$", s_nocode):
        ids.add(slug(m.group(1)))
    anchors[f] = ids
    texts[f] = s_nocode

broken = 0
for f in files:
    for m in re.finditer(r"\[[^\]]*\]\(([^)\s]+)\)", texts[f]):
        target = m.group(1)
        if re.match(r"^[a-z][a-z0-9+.-]*:", target):
            continue
        path, _, frag = target.partition("#")
        dest = f if not path else os.path.normpath(os.path.join(os.path.dirname(f), path))
        if path and not os.path.exists(os.path.join(root, dest)):
            print(f"{f}: missing file {target}"); broken += 1; continue
        if frag and dest.endswith(".md") and dest in anchors and frag not in anchors[dest]:
            print(f"{f}: missing anchor {target}"); broken += 1
print(f"{broken} broken")
