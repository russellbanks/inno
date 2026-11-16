# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

A Cargo workspace with two crates:

- **`core/`**: the `inno` library, published to crates.io. It is a read-only parser for Inno Setup installer `.exe` files and supports Inno Setup versions up to 7.1.x (`MAX_SUPPORTED_VERSION` in `core/src/lib.rs`). `#![forbid(unsafe_code)]`.
- **root package (`cli/src/main.rs`)**: `innex`, a ratatui TUI and extractor built on `inno`. It serves as the reference consumer of the library. It is not published.

Both crates share their version, edition (2024), and MSRV (1.93) through `[workspace.package]`. The project's reference implementations are innoextract and the Inno Setup source (`jrsoftware/issrc`). Code comments often link to specific `issrc` tags, and fixes are often described as matching innoextract's behavior.

## Commands

```sh
cargo build                                   # builds innex + inno
cargo run -- path/to/setup.exe                # TUI viewer
cargo run -- path/to/setup.exe --debug        # dump {:#?} of the whole parsed structure
cargo run -- setup.exe -e outdir [-f REGEX]   # extract files (regex matched against destination)

cargo test --workspace --all-features         # unit + doc tests (fast, offline)
cargo test -p inno <name>                     # single test by name filter
cargo test -p inno --all-features -- --ignored              # integration tests (download real installers)
cargo test -p inno --test inno_versions -- --ignored        # one integration test file

cargo clippy                                  # CI runs with RUSTFLAGS=-D warnings
cargo +nightly fmt                            # rustfmt.toml uses unstable options; stable fmt ignores them
```

CI (`.github/workflows/ci.yml`) also runs:
- `cargo hack check --feature-powerset --optional-deps` and `cargo hack test --workspace --feature-powerset --optional-deps`. Code must compile under every combination of the `chrono`, `jiff`, and `extract` features.
- `cargo hack --workspace msrv verify`, `cargo minimal-versions check --workspace`, and `cargo-machete`, which checks for unused dependencies.
- `cargo publish --package inno --dry-run`

Pushing a `v*` tag publishes `inno` to crates.io (`release.yml`).

### Integration tests
Everything in `core/tests/` is `#[ignore]`d because these tests download official Inno Setup installers at runtime (`tests/common/mod.rs::download_inno_version`, which builds GitHub or jrsoftware.org URLs from a semver string). `inno_versions.rs` is an `rstest` matrix over every supported release. Add each new Inno Setup release to that list. Test files that need `extract` start with `#![cfg(feature = "extract")]` so the feature powerset still compiles.

## Architecture (core)

### Parse pipeline: `Inno::new` in `core/src/lib.rs`
1. **`loader::SetupLoader`**: walks the PE (`pe/`: DOS, COFF, and optional headers, section table, resource directory) to find the setup loader resource. That resource gives the header and data offsets.
2. **`InnoVersion::read`**: reads the 64-byte version string (`"Inno Setup Setup Data (x.y.z) (u)"`) and derives the variant flags (`UNICODE`, `ISX`, `BITS_16`).
3. **Version ambiguity retry**: some Inno releases changed the on-disk format without bumping the version string. `InnoVersion::ambiguous_candidates()` lists alternative versions. `Inno::new` rewinds the reader and re-parses with each candidate until one succeeds.
4. **`InnoInner::read_stream`**: for 6.5 and later, reads an `EncryptionHeader` first. It then opens an `InnoStreamReader` (`read/stream.rs`), which checks the CRC'd stream header, picks LZMA1, zlib, or stored compression, and reads through `read/block.rs` (4 KiB CRC32-checked blocks) and `read/decoder.rs`. The **primary** header stream holds `Header`, then languages, then every entry list in a fixed order (messages, permissions, types, components, tasks, dirs, ISSigKeys, files, icons, ini, registry, install/uninstall delete, install/uninstall run), then `Wizard`. Before 4.0 the wizard comes before messages. `reader.reset()` then opens the **secondary** stream of `FileLocation`s. Both streams must end exactly (`check_end_of_stream`). Leftover bytes almost always mean a field was read at the wrong version.
5. **Text decoding**: Unicode installers use UTF-16LE. ANSI installers use a codepage taken from the language entries, with Windows-1252 as the fallback. `Header::decode(codepage)` runs after the languages are read.

### Version-gated field reading (the core pattern)
Every entry type in `core/src/entry/` and the header in `core/src/header/` has a `read(reader, codepage, version)` that mirrors the Inno Setup struct for that version. Fields are added, removed, or widened according to version comparisons. `InnoVersion` implements `PartialOrd` against `u8`, `f32`, and tuples, so all of these work:

```rust
if version >= (4, 0, 1) { ... }          // tuple
if version >= 6.7 { ... }                // f32 → (major, minor)
if version >= 4 || (version.is_isx() && version >= (1, 3, 24)) { ... }
```

Flag sets are packed bitfields whose membership depends on the version. Use the `read_flags!` macro (`header/flag_reader.rs`) and list flags in the exact order Inno Setup declares them, with `if version >= ... => FLAG` for conditional ones. Order matters because each flag occupies the next bit. 3-byte bitfields are padded to 4 bytes.

Primitive reads go through `read::ReadBytesExt` (`read_u32::<LE>()`, `read_t::<T: FromBytes>()`, `read_decoded_pascal_string(codepage)`). Many fixed-layout structs derive `zerocopy` traits.

Adding support for a new Inno Setup version usually means: bump `MAX_SUPPORTED_VERSION`, compare the `issrc` struct definitions against the previous tag, add version-gated fields or flags to the affected `read` functions, and add the release to `tests/inno_versions.rs`.

### Extraction (`extract` feature)
`Inno::files()`, `filtered_files(pred)`, and `streaming_files(pred)` (in `core/src/iterator/`) read file data from the executable (`read::Embedded`) or from `.bin` slices next to it (`slice::Slices`; use `Inno::open(path)` or `.located_at(path)` so they can be found). `read/data_chunk.rs` and `read/chunk/` handle per-chunk compression (zlib, LZMA1, LZMA2), encryption (`encryption/`), and the CALL/JMP instruction filters (`entry/location/compression_filter.rs`). Checksums (CRC32, Adler32, MD5, SHA-1, SHA-256 in `entry/checksum/`) are verified as data is read. Many files can share one `FileLocation`, so iterators must not de-duplicate entries by location.

`Inno` methods are thin delegates to `InnoInner`. When adding an accessor, add it to both.

## Architecture (cli)
`cli/src/main.rs` handles `--debug` and `--extract`, and otherwise starts the TUI. `tabs.rs` and `page.rs` define a `Page` enum with one variant per entry type. Each module (`files.rs`, `registries.rs`, and so on) is a table widget with `next_row`/`previous_row`. A new page needs a `Page` variant plus matching arms in `App::scroll_up`, `App::scroll_down`, and `render`.

## Style
- `rustfmt.toml`: `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"` (std / external / crate groups). Requires nightly rustfmt.
- Public getters are usually `#[must_use] #[inline] pub const fn` returning `Option<&str>` or slices. Doc comments use `#[doc(alias = "...")]` to give the original Inno Setup (Pascal) field name.
