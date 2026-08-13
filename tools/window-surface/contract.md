# /home/j/Repos/thaum-renderer/tools/window-surface

## purpose
Own reusable helper seams for creating and managing the real graphics window and GPU presentation surface consumed by thaum-renderer.

## owns
- cross-platform renderer window-surface helper organization
- reusable window creation and presentation-surface setup helpers
- low-level presentation concerns shared by renderer runtime paths

## does not own
- renderer design truth
- app-specific window chrome or UX policy
- post-effect semantics
- cell composition behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/tools/`

## exposed interfaces
- window-surface helpers
  - describes the reusable helper seam for preparing a real window and GPU-backed presentation target for renderer output

## interface consumers
- future renderer runtime surfaces
- future test scenes

## artifacts
- none

## tests
- none

## data
- none

## notes
- the renderer should present to a real window with real graphics rather than treating png output or terminal output as the primary target
- the preferred implementation direction is a GPU-first cross-platform surface suitable for Linux, Windows, Apple platforms, and Android-class targets
- in Rust, `wgpu` plus `winit` is the current best-fit direction for the primary implementation path unless later research proves a better cross-platform fit
