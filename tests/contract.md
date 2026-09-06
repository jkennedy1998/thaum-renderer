# thaum-renderer/tests

## purpose
Own renderer-local proof tests and smoke checks for thaum-renderer without becoming a second app launcher surface.

## owns
- renderer-local proof test declarations
- renderer-local test-only helpers when a proof needs them
- repo-local smoke coverage for compile/build behavior

## does not own
- the canonical manual proof app
- app-shaped scene development
- a second bootable test executable

## children-encapsulations
- none

## contents
- `contract.md`
  - contract for the renderer-local tests seam

## dependencies
- `thaum-renderer/`
- `thaum-renderer-test-user/` as the consumed external manual proof app

## exposed interfaces
- none

## interface consumers
- humans and operators validating thaum-renderer

## artifacts
- none

## tests
- `workspace-build-smoke`
  - light
  - proves the renderer workspace compiles after local renderer changes.

## data
- none

## notes
- the canonical manual proof executable is `thaum-renderer-test-user/target/release/thaum-renderer-test-user`
- renderer-local `tests/` should hold proof tests, not a second consumer app
