# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- [ ] Keyed dependencies
- [ ] Async dependencies
- [ ] Factory dependencies
- [ ] DSL for registering dependencies

## [0.3.1](https://github.com/WarriorsSami/yadir/compare/yadir-v0.3.0...yadir-v0.3.1) - 2024-07-21

### Other
- *(yadir_derive)* report syn::Error instead of panicking
- *(yadir)* show complete examples for rustdocs in di manager and update yadir version in README.md

## [0.3.0](https://github.com/WarriorsSami/yadir/compare/yadir-v0.2.3...yadir-v0.3.0) - 2024-07-20

### Added
- *(yadir)* add lifetime specifiers (singleton, transient) and improve registering/resolving deps api for di manager

### Other
- *(yadir)* update rustdocs to correspond to latest version of yadir_derive
- *(yadir)* add unreleased features to CHANGELOG.md
- [x] Enhanced DI Manager API for registering and resolving dependencies
- [x] Lifetime specifiers for dependencies, mainly singleton and transient ones

## [0.2.3](https://github.com/WarriorsSami/yadir/compare/yadir-v0.2.2...yadir-v0.2.3) - 2024-07-20

### Fixed
- *(yadir)* correct deps! and let_deps! macros matching rules to accept more than 3 "csv"

## [0.2.2](https://github.com/WarriorsSami/yadir/compare/yadir-v0.2.1...yadir-v0.2.2) - 2024-07-20

### Added
- *(proc_macro)* add support for specifying the output type for a struct when deriving DIBuilder
- add derive proc macro for DIBuilder which identifies input deps from struct definition fields

### Other
- update README according to the new versions of yadir and yadir_derive

## [0.2.1](https://github.com/WarriorsSami/yadir/compare/v0.2.0...v0.2.1) - 2024-07-16

### Other
- Merge remote-tracking branch 'origin/master'
- update readme to showcase newly added macros usage

## [0.2.0](https://github.com/WarriorsSami/yadir/compare/v0.1.0...v0.2.0) - 2024-07-16

### Added
- [**breaking**] add declarative macros for easing dependencies declarations and destructuring

### Other
- add link to crates.io on version badge
- Merge remote-tracking branch 'origin/master'
- add badges for crates.io version and ci/cd status

## [0.1.0](https://github.com/WarriorsSami/yadir/releases/tag/v0.1.0) - 2024-07-15

### Added
- add di primitives and tests

### Fixed
- specify correct path to license file in Cargo.toml

### Other
- fill cargo manifest fields and add pipeline for auto releasing/publishing to crates.io
- remove typo from readme
- add rustdocs and general info in readme
- Initial commit
