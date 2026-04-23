#!/usr/bin/env python3
"""Dump raw text from first 8 pages of HSV PDF for format analysis.

Usage:
    nix-shell -p python3Packages.pymupdf --run 'python tools/peek_pdf.py HSV_Bijbel.pdf' | tee /tmp/hsv_peek.txt
"""
import sys
import fitz

path = sys.argv[1] if len(sys.argv) > 1 else "HSV_Bijbel.pdf"
doc = fitz.open(path)
for i, page in enumerate(doc):
    if i >= 8:
        break
    print(f"=== PAGE {i+1} ===")
    for line in page.get_text().split('\n'):
        line = line.strip()
        if line:
            print(repr(line))
    print()
doc.close()
