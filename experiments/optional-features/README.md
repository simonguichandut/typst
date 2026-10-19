# Optional library features for mathematical plot labels

Experimental branch based on **Typst v0.14.2**. This explores using Typst for
Kuva's mathematical labels without compiling bibliography processing, code syntax
highlighting, or WebAssembly plugins. It is a feasibility experiment, not a
proposal to merge or a claim of full Typst compatibility testing.

`typst-library` now has three optional capabilities, all **enabled by default**:

| Feature | Optional dependencies | Behavior when disabled |
| --- | --- | --- |
| `bibliography` | `hayagriva` | Bibliographies and citations report an explicit error; ordinary document references remain available. |
| `syntax-highlighting` | `syntect`, `two-face` | Raw text retains plain rendering, preprocessing and line numbering; custom syntax/theme loading reports an error. |
| `plugins` | `wasmi` | Plugin loading and calls report an explicit error. |

The enabled bibliography and plugin implementations are unchanged. Public API
shells keep the types required by evaluation/layout; APIs whose signatures expose
removed dependency types are unavailable with their feature disabled.

The eight compiler/layout/export crates forward their own defaults to the library
but disable dependency defaults internally. CLI, IDE, kit, PDF export, docs and
upstream test consumers explicitly retain defaults. To use the slim configuration,
the consumer must disable defaults on **every** direct Typst dependency; Cargo
feature unification can otherwise re-enable the capabilities. Individual features
are currently selected through `typst-library`, without facade aliases. A full
workspace build intentionally retains normal defaults.

## Results from the original local prototype

Kuva 0.6.0, Typst 0.14.2, Rust 1.98.0, four Cargo jobs, no incremental compilation,
and the upstream Kuva development profiles. These were sequential, separate cold
build directories with an already downloaded dependency cache on a busy machine.
One observation per configuration; not a controlled statistical benchmark.

| Measurement | Full | Slim |
| --- | ---: | ---: |
| Active external normal/build package versions | 319 | 267 |
| Clean development build | 231.18 s | 180.92 s |
| User + system CPU | 633.72 s | 509.06 s |
| Development CLI binary | 165.21 MiB | 132.79 MiB |
| Build directory before tests | 1.430 GiB | 1.196 GiB |

This removed **52 packages**, about one quarter of the 206-package increase from
enabling Typst in that Kuva configuration. Optional packages remain in Cargo.lock;
lockfile entries are not the active compiled dependency graph. Measurements are for
development builds of standalone Kuva, not release builds or application-wide CI.
They were obtained with patched published-crate manifests before this workspace
port; the workspace port is verified separately, not re-benchmarked.

The [workspace port validation](workspace-validation.json) passed the same 21 math
tests and nine capability probes in each configuration, plus all 36 CLI figure
comparisons. Its resolved dependency graphs also contain 319/267 external packages.

Each configuration passed 21 enabled Kuva math tests and nine capability probes.
The probes include plain raw text, ordinary heading references, disabled-capability
errors, and restoration of valid WASM loading and bibliography rendering. Four
lookup-only test cases are compiled out when `typst-math` is enabled.

Six labels were rendered as SVG, PNG and PDF using original Kuva, patched full,
and patched slim: 54 artifacts and 36 comparisons. PNG bytes/pixels and PDF bytes
matched; SVG matched after canonicalizing generated glyph IDs. No custom renderer
or mathematical layout replacement was used.

- [Original/full/slim gallery](renders/index.html) (open locally after cloning).
- [Original Ramsey PNG](renders/original-0.png) / [slim Ramsey PNG](renders/slim-0.png).
- [Numeric summary](summary.json), [render results](render-comparison.json),
  [validation stages](validation-results.json), and [dependency graphs](slim-tree.txt).

## Reproduce against this checkout

Requires Python 3, Cargo/Rust and network access for dependencies. Rust 1.98.0 was
used for the recorded checks; upstream's minimum supported version was not tested.
Install that toolchain with `rustup toolchain install 1.98.0` if needed.
The scripts use only Python's standard library. Run from the fork checkout:

```sh
python3 experiments/optional-features/prepare.py /tmp/typst-kuva-experiment
RUSTUP_TOOLCHAIN=1.98.0 python3 experiments/optional-features/verify.py /tmp/typst-kuva-experiment
```

`prepare.py` downloads the checksum-pinned Kuva 0.6.0 archive, sets its direct Typst
dependencies to `default-features = false`, adds local path patches, and installs
the capability probe. Fonts and their licenses come from Kuva's archive. The work
directory must be absent or empty. An existing archive can be passed with
`--archive /path/to/kuva-0.6.0.crate`.

`verify.py` runs the math tests and capability probes with all three features off
and on, builds the CLI, records the resolved graphs, and checks the figures
against the saved original outputs. It writes logs and images under the work
directory's `verification/`. This uses normal Cargo caching and is **not** a
benchmark. Strict output equality is a check for this pinned experiment, not a
promise of byte-identical output across arbitrary platforms/toolchains.

The relevant Cargo feature selections are:

```sh
# Slim: direct Typst dependency defaults disabled by prepare.py.
cargo build --manifest-path /tmp/typst-kuva-experiment/kuva-0.6.0/Cargo.toml \
  --no-default-features --features cli,full,typst-math --bin kuva

# Restore all three capabilities.
cargo build --manifest-path /tmp/typst-kuva-experiment/kuva-0.6.0/Cargo.toml \
  --no-default-features --features cli,full,typst-math,typst-library/default --bin kuva
```

For clean-build measurements, use a different initially empty `CARGO_TARGET_DIR`
for each configuration, keep the toolchain/jobs/profiles identical, prefetch the
dependencies, and record CPU time as well as wall time. Do not interpret warm
verification builds as clean-build timings.

## Scope remaining

Publication commits skip the broad GitHub CI matrix; the local checks described
here are the validation performed for this experiment.

The full upstream Typst test matrix and all independent feature combinations have
not been validated. There are no release-build measurements. This branch does not
include the separate custom-exporter/SVG-rasterizer experiments, and it does not
claim to remove the majority of the Typst engine: 154 additional active packages
remain relative to the original non-Typst Kuva baseline. No upstream issue or pull
request is part of this experiment.
