#!/usr/bin/env python3
"""Check selected copy rules in the Rust egui interface and documentation.

Usage: python3 scripts/check-design-system.py [file ...]
This is a narrow text check, not a Rust parser or a substitute for visual review.
"""
from pathlib import Path
import re
import sys

FILES = ('src/main.rs', 'src/ascii.rs', 'src/io.rs', 'index.html', 'README.md')
# Protocol, format magic and decoder error messages are not labels.
EXEMPT = {'GET', 'RIFF', 'CMYK JPEG'}
uppercase_string = re.compile(r'(?<![A-Za-z0-9_])"([A-Z][A-Z0-9 :/\.\-…×]{2,})"')
failures = []
for name in sys.argv[1:] or (*FILES, *(str(p) for p in Path('docs/system').rglob('*.md'))):
    path = Path(name)
    for number, line in enumerate(path.read_text(encoding='utf-8').splitlines(), 1):
        if '—' in line:
            failures.append((name, number, 'no-em-dash'))
        if path.suffix == '.rs':
            for match in uppercase_string.finditer(line):
                text = match.group(1)
                if text not in EXEMPT and sum(ch.isalpha() for ch in text) >= 2:
                    failures.append((name, number, 'no-uppercase-label'))
        elif path.suffix == '.html':
            for match in re.finditer(r'<title>(.*?)</title>', line):
                if re.search(r'\b[A-Z]{2,}\b', match.group(1)):
                    failures.append((name, number, 'no-uppercase-label'))
for name, number, rule in failures:
    print(f'{name}:{number}: {rule}')
sys.exit(bool(failures))
