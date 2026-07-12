# /home/j/Repos/thaum-renderer/domain/cell-shader/custom-shaders

## purpose
Own the optional user-authored shader seam expected by thaum-renderer outside the renderer core.

## owns
- the contract boundary for user-authored custom shaders
- the rule that renderer consumers can provide their own shader definitions in the expected format
- the seam between renderer-known shader ids and consumer-provided shader behavior

## does not own
- built-in renderer core logic by itself
- shader data intake ownership
- app-side gameplay logic

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`

## exposed interfaces
- custom shader shape
  - describes the expected authoring and registration boundary for consumer-provided shaders

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- renderer should expect one optional user place for custom shader ownership rather than hard-coding every shader into core
