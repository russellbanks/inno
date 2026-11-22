# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `inno::path` module with `InnoPath`, a parsed Inno Setup path template such as
  `{app}\bin\foo.dll`. `InnoPath::to_relative_path` resolves constants through a `ResolveConstant`
  (`ByName`, or a closure via `path::from_fn`). It returns a `RelativePath` that is safe to join onto
  a directory, and rejects `..`, absolute paths, invalid characters and reserved names such as `CON`
  with a `PathError`. ([#223])
- With the `extract` feature, BZip2-compressed file data can be extracted. This covers both
  installers whose header specifies BZip2 and the per-file `BZIPPED` flag used before 4.0.1.
  Decoding uses the `bzip2` crate's pure-Rust libbzip2 port, so it does not need a C toolchain.
  ([#226])

### Changed

- **Breaking:** These getters now return `InnoPath` instead of `&str`: `File::destination`,
  `Directory::name`, `Icon::name`, `Icon::filename`, `Ini::file_path`, `DeleteEntry::name` and
  `RunEntry::name`. ([#223])
- **Breaking:** `Compression` now defaults to `Stored`. Headers from 4.1.5 to 4.2.5 with an
  unrecognised compression byte now fail with `InvalidData`, as 4.2.6 and later already did.
  ([#226])

### Removed

- **Breaking:** `File::normalized_destination`. Use `File::destination()` and then
  `InnoPath::to_relative_path`. ([#223])
- **Breaking:** `InnoError::UnsupportedCompression`, `Compression::Unknown`, and
  `From<u8> for Compression`. Every compression method can now be decoded. ([#226])
- **Breaking:** `PartialEq` and `PartialOrd` between `InnoVersion` and `f32`. Compare against
  `(major, minor)` tuples instead, for example `version >= (6, 5)`. The `f32` impls decoded the
  minor version lossily and supported only single-digit minors. ([#227])

### Security

- A corrupt or malicious installer can no longer abort the process by declaring a string or wizard
  data length of up to 4 GiB. These buffers now grow in 10 KiB chunks as bytes arrive, as
  innoextract does, and reading fails with `UnexpectedEof` if the input ends early.

[Unreleased]: https://github.com/russellbanks/inno/compare/v0.7.6...HEAD
[#223]: https://github.com/russellbanks/inno/pull/223
[#226]: https://github.com/russellbanks/inno/pull/226
[#227]: https://github.com/russellbanks/inno/pull/227
