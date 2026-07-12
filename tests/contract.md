# /home/j/Repos/thaum-renderer/tests

## purpose
Own consumer-shaped validation seams for thaum-renderer.

## owns
- renderer-facing test consumers and scenes
- lightweight scenarios that help prove the contract bounds of the renderer
- manual visual validation seams where a human confirms the rendered output looks right

## does not own
- renderer production ownership
- app gameplay logic

## children-encapsulations
- `thaum-renderer-test-scene/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/`
- `/home/j/Repos/thaum-renderer/orchestration/`

## exposed interfaces
- none

## interface consumers
- humans and operators shaping thaum-renderer
- future automated tests

## artifacts
- none

## tests
- `thaum-renderer-test-scene`
  - planned
  - validates a broad boot scene with manual visual confirmation over sprite, glyph, material, and composition behavior.

## data
- none

## notes
- test consumers are acceptable places to define module-shaped or scene-shaped examples when the goal is validation rather than renderer ownership
