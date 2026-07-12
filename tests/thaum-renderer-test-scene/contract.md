# /home/j/Repos/thaum-renderer/tests/thaum-renderer-test-scene

## purpose
Own a manual validation scene for booting thaum-renderer and checking that current renderer contracts resolve correctly.

## owns
- the renderer test-scene seam
- a bootable validation surface for sprite, glyph, material, and composition behavior
- manual user-confirmed visual validation for current renderer expectations
- inclusion of grayscale material coverage for early material testing

## does not own
- renderer production ownership
- app-specific module semantics outside this test seam
- gameplay logic

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/`
- `/home/j/Repos/thaum-renderer/orchestration/`

## exposed interfaces
- test scene shape
  - describes a minimal but broad scene used to boot renderer paths and visually confirm correct output

## interface consumers
- `/home/j/Repos/thaum-renderer/tests/`
- humans and operators shaping thaum-renderer
- future automated tests with manual pass support

## artifacts
- none

## tests
- none

## data
- none

## notes
- a true pass currently includes a human confirming that the rendered output looks right
