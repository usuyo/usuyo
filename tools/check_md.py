"""Mechanical Markdown check for one file (argv[1]).

- every ATX heading outside code has an explicit <a id> on the previous non-blank line,
  equal to the GitHub slug of the heading text
- duplicate explicit ids
- relative links (incl. <...> form) resolve to an existing file; #frag resolves to an
  explicit <a id> in the target (heading-slug-only matches are reported as SLUGONLY)
- bare §N / 第 N 条 outside links and code
- banned words (whole file; caller filters to added lines)
"""
import os, re, sys, unicodedata

def gh_slug(text):
    # GitHub: strip markdown inline markup roughly, lowercase, drop punctuation, spaces->-
    t = re.sub(r'`([^`]*)`', r'\1', text)
    t = re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', t)
    t = re.sub(r'<[^>]+>', '', t)
    t = t.strip().lower()
    out = []
    for ch in t:
        cat = unicodedata.category(ch)
        if ch == ' ':
            out.append('-')
        elif ch in '-_':
            out.append(ch)
        elif cat[0] in 'LN' or cat in ('Mn', 'Mc'):
            out.append(ch)
    return ''.join(out)

def scan(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    in_code = False
    code = []
    for line in lines:
        if re.match(r'^\s*(```|~~~)', line):
            code.append(True); in_code = not in_code; continue
        code.append(in_code)
    return lines, code

def explicit_ids(path):
    lines, code = scan(path)
    ids = []
    for l, c in zip(lines, code):
        if c: continue
        ids += re.findall(r'<a\s+(?:id|name)="([^"]+)"', l)
    return ids

def heading_slugs(path):
    lines, code = scan(path)
    s = set()
    for l, c in zip(lines, code):
        if c: continue
        m = re.match(r'^(#{1,6})\s+(.*?)\s*#*\s*$', l)
        if m: s.add(gh_slug(m.group(2)))
    return s

BANNED = ['critical','essential','comprehensive','robust','seamless','powerful','elegant',
          'leverage','delve','in conclusion','至关重要','核心','全面','强大','优雅','赋能','打磨','精心',
          '破解','去水印','绕过','知识库','第二大脑','backlog']

def main(path):
    errs = []
    path = os.path.abspath(path)
    base = os.path.dirname(path)
    lines, code = scan(path)
    ids = explicit_ids(path)
    seen = set()
    for i in ids:
        if i in seen: errs.append(f'DUPID {i}')
        seen.add(i)
    prev = ''
    for n, (l, c) in enumerate(zip(lines, code), 1):
        if c: continue
        m = re.match(r'^(#{1,6})\s+(.*?)\s*#*\s*$', l)
        if m:
            want = gh_slug(m.group(2))
            pm = re.fullmatch(r'\s*<a\s+id="([^"]+)"\s*></a>\s*', prev)
            if not pm:
                errs.append(f'L{n} NOANCHOR heading: {l.strip()[:80]}')
            elif pm.group(1) != want:
                errs.append(f'L{n} SLUGMISMATCH id={pm.group(1)!r} slug={want!r}')
        if l.strip(): prev = l
    cache = {}
    for n, (l, c) in enumerate(zip(lines, code), 1):
        if c: continue
        lc = re.sub(r'`[^`]*`', '', l)
        targets = re.findall(r'\]\(<([^>]+)>\)', lc) + re.findall(r'\]\(([^)<\s]+)(?:\s+"[^"]*")?\)', lc)
        targets += re.findall(r'href="([^"]+)"', lc)
        for t in targets:
            if re.match(r'(https?|mailto|ftp):', t): continue
            fp, _, frag = t.partition('#')
            from urllib.parse import unquote
            fp = unquote(fp); frag = unquote(frag)
            tgt = os.path.normpath(os.path.join(base, fp)) if fp else path
            if not os.path.exists(tgt):
                errs.append(f'L{n} MISSINGFILE {t}'); continue
            if frag:
                if os.path.isdir(tgt) or not tgt.endswith('.md'):
                    errs.append(f'L{n} FRAG-ON-NONMD {t}'); continue
                if tgt not in cache: cache[tgt] = (set(explicit_ids(tgt)), heading_slugs(tgt))
                eid, hs = cache[tgt]
                if frag not in eid:
                    if frag in hs: errs.append(f'L{n} SLUGONLY {t}')
                    else: errs.append(f'L{n} MISSINGANCHOR {t}')
        st = re.sub(r'\[[^\]]*\]\([^)]*\)', 'LINK', lc)
        for m in re.finditer(r'§\s?\d+(\.\d+)*', st):
            errs.append(f'L{n} BARE {m.group(0)} …{st[max(0,m.start()-20):m.end()+8]}')
        for m in re.finditer(r'第\s?\d+([–\-~]\d+)?(、\d+)*\s?(条|节|章)', st):
            errs.append(f'L{n} BARE {m.group(0)} …{st[max(0,m.start()-20):m.end()+8]}')
    for n, l in enumerate(lines, 1):
        for w in BANNED:
            for m in re.finditer(re.escape(w), l, re.I):
                errs.append(f'L{n} BANNED {w}: …{l[max(0,m.start()-25):m.end()+15]}')
    for e in errs: print(e)
    print(f'{len(errs)} issues in {os.path.relpath(path)}')

main(sys.argv[1])
