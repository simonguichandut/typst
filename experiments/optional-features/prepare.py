#!/usr/bin/env python3
"""Prepare an isolated Kuva 0.6 consumer of this local Typst checkout."""
import argparse
import gzip
import hashlib
from pathlib import Path
import shutil
import tarfile
import urllib.request

HERE = Path(__file__).resolve().parent
FORK = HERE.parents[1]
CRATES = ('typst', 'typst-eval', 'typst-html', 'typst-layout', 'typst-library',
          'typst-realize', 'typst-render', 'typst-svg')
SHA256 = '32a778113ce802ab7bc34eed1a47e20b876ac7f0eb2c4ba88de31acac55af0e9'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('work_dir', type=Path)
parser.add_argument('--archive', type=Path, help='Use an already downloaded kuva-0.6.0.crate')
args = parser.parse_args()
work = args.work_dir.resolve()
if work.exists() and any(work.iterdir()):
    parser.error('work_dir must be absent or empty')
work.mkdir(parents=True, exist_ok=True)
archive = work / 'kuva-0.6.0.crate'
if args.archive:
    shutil.copyfile(args.archive, archive)
else:
    urllib.request.urlretrieve('https://crates.io/api/v1/crates/kuva/0.6.0/download', archive)
if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
    raise SystemExit('Unexpected Kuva archive checksum')
with tarfile.open(archive) as tar:
    for member in tar.getmembers():
        path = Path(member.name)
        if path.is_absolute() or '..' in path.parts or not (member.isfile() or member.isdir()):
            raise SystemExit(f'Unsupported archive entry: {member.name}')
    tar.extractall(work)
source = work / 'kuva-0.6.0'
manifest = source / 'Cargo.toml'
text = manifest.read_text()
for name in CRATES:
    text = text.replace(f'[dependencies.{name}]\n', f'[dependencies.{name}]\ndefault-features = false\n')
# TOML literal strings keep filesystem backslashes literal on Windows.
text += '\n[patch.crates-io]\n'
for name in CRATES:
    path = str(FORK / 'crates' / name)
    if "'" in path:
        raise SystemExit('Checkout paths containing apostrophes are not supported by this experiment')
    text += f"{name} = {{ path = '{path}' }}\n"
text += '\n[[example]]\nname = "capability_probe"\npath = "examples/capability_probe.rs"\nrequired-features = ["typst-math"]\n'
manifest.write_text(text)
shutil.copyfile(HERE / 'capability_probe.rs', source / 'examples/capability_probe.rs')
fonts = work / 'fonts'
fonts.mkdir()
for name in ('DejaVuSans.ttf', 'NewCMMath-Regular.otf'):
    (fonts / name).write_bytes(gzip.decompress((source / 'assets/fonts' / (name + '.gz')).read_bytes()))
for name in ('LICENSE', 'LICENSE-NewCM'):
    shutil.copyfile(source / 'assets/fonts' / name, fonts / name)
print(manifest)
