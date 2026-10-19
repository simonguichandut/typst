#!/usr/bin/env python3
"""Run Kuva math tests and compare full/slim figures with saved original output.

Uses normal Cargo caching. This is verification, NOT a clean-build benchmark.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('work_dir', type=Path, help='Directory populated by prepare.py')
args = parser.parse_args()
work = args.work_dir.resolve()
source = work / 'kuva-0.6.0'
out = work / 'verification'
out.mkdir(exist_ok=True)
env = dict(os.environ)
env.setdefault('CARGO_BUILD_JOBS', '4')
env.setdefault('CARGO_INCREMENTAL', '0')
env.setdefault('CARGO_TARGET_DIR', str(work / 'target'))
target = Path(env['CARGO_TARGET_DIR']).resolve()
env['CARGO_TARGET_DIR'] = str(target)
labels = [r'Ramsey $T_2^*$', r'Population $\frac{1}{\sqrt{2}}$',
          r'Decay $A e^{-t/T_1}$', r'Frequency $\omega = 2\pi f$',
          r'Root $\sqrt[3]{x+1}$', r'Phase $\phi = \pi/2$']
results = []
def run(name, command):
    with (out / (name + '.log')).open('w') as log:
        subprocess.run(command, cwd=source, env=env, stdout=log,
                       stderr=subprocess.STDOUT, check=True)
def normalize(svg):
    ids = {}
    return re.sub(r'g[0-9A-F]{20,}', lambda m: ids.setdefault(m[0], f'g{len(ids)}'), svg)
for variant in ('slim', 'full'):
    features = 'cli,full,typst-math' + (',typst-library/default' if variant == 'full' else '')
    flags = ['--manifest-path', str(source / 'Cargo.toml'), '--no-default-features', '--features', features]
    run(variant + '-tests', ['cargo', 'test', *flags, '--test', 'math_lookup',
                           '--test', 'math_smoke', '--test', 'math_png', '--test', 'math_pdf'])
    run(variant + '-capabilities', ['cargo', 'run', *flags, '--example', 'capability_probe', '--', variant])
    run(variant + '-build', ['cargo', 'build', *flags, '--bin', 'kuva'])
    run(variant + '-tree', ['cargo', 'tree', *flags, '--edges', 'normal,build', '--prefix', 'none', '--format', '{p}'])
    binary = target / 'debug' / ('kuva.exe' if os.name == 'nt' else 'kuva')
    for index, label in enumerate(labels):
        for ext in ('svg', 'png', 'pdf'):
            image = out / f'{variant}-{index}.{ext}'
            p = subprocess.run([str(binary), 'scatter', str(HERE / 'points.tsv'), '--x', 'x', '--y', 'y',
                                '--title', label, '--x-label', r'Delay $\tau$ (us)',
                                '--y-label', r'Population $\frac{1}{2}$', '-o', str(image)],
                               env=env, capture_output=True, text=True, check=True)
            if p.stderr:
                raise RuntimeError(p.stderr)
            reference = HERE / 'renders' / f'original-{index}.{ext}'
            equal = (normalize(image.read_text()) == normalize(reference.read_text()) if ext == 'svg'
                     else image.read_bytes() == reference.read_bytes())
            results.append({'variant': variant, 'label': label, 'format': ext, 'identical': equal})
(out / 'render-comparison.json').write_text(json.dumps(results, indent=2))
assert all(result['identical'] for result in results), 'Rendering differs; inspect verification outputs'
print(f'Passed both test/probe configurations and all {len(results)} render comparisons.')
