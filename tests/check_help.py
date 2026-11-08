#!/usr/bin/env python3
"""Validate the shipped static help catalogue and every relative reference."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
HELP = ROOT / 'dist' / 'help'
ORIGINAL = ROOT.parent / 'analysis/ipa/Payload/Sudoku 401.app/help'

class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.refs = []
        self.lang = None
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'html': self.lang = attrs.get('lang')
        if tag in ('a','img','link'):
            ref = attrs.get('href') or attrs.get('src')
            if ref: self.refs.append(ref)
        if tag == 'img': assert attrs.get('alt'), 'image has no alt text'

pages = sorted(HELP.glob('*.html'))
assert {p.name for p in pages} == {p.name for p in ORIGINAL.glob('*.html')}, 'original help page missing'
assert len(pages) == 25, len(pages)
for page in pages:
    parser = Links()
    parser.feed(page.read_text(encoding='utf-8'))
    assert parser.lang == 'zh-CN', page
    for ref in parser.refs:
        url = urlsplit(ref)
        assert not url.scheme and not url.netloc and not ref.startswith('/'), (page,ref)
        if url.path:
            target = (page.parent / unquote(url.path)).resolve()
            assert target.is_file(), (page, ref)
index = Links()
index.feed((HELP / 'index.html').read_text(encoding='utf-8'))
assert {ref for ref in index.refs if ref.endswith('.html') and ref not in ('index.html', '../index.html')} == {p.name for p in pages if p.name != 'index.html'}
for page in pages:
    text = page.read_text(encoding='utf-8')
    assert '译注' not in text and '资料翻译并整理' not in text
assert 'background:#fff' in (HELP / 'help.css').read_text(encoding='utf-8')
print(f'{len(pages)-1} Chinese technique pages, index and all local links/images verified')
