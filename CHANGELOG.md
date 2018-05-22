# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## 1.0.3 - 2018-05-21
### Fixed
- Use global path for Result, Option, Box in macro expansion to avoid name
  conflicts with locally-defined symbols.

## 1.0.2 - 2018-05-12
### Added
- Support for downcasting `Box<Trait>` to `Box<Concrete>`.

## 1.0.1 - 2018-03-08
### Fixed
- Don't use types as traits in macros.

## 1.0.0 - 2017-02-05
### Changed
- Cleaned up README and published 1.0.

## 0.1.2 - 2016-11-22
### Added
- Support for associated types as well.

## 0.1.1 - 2016-09-04
### Added
- Downcast functionality to downcast borrowed mutable and immutable trait
  objects to concrete types. Supports concrete type parameters and type
  variables with optional constraints.

