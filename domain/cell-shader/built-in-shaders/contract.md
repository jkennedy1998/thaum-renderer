# /home/j/Repos/thaum-renderer/domain/cell-shader/built-in-shaders

## purpose
Own the small renderer-provided built-in shader set used to prove the shader system shape.

## owns
- built-in shader placement inside renderer
- the minimal standard shader set provided by renderer

## does not own
- app-owned custom shaders
- renderer-wide data-lane ownership
- shader stack ordering outside the shared cell-shader contract

## children-encapsulations
- `weight-sin/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`

## exposed interfaces
- built-in shader family
  - describes the small renderer-authored shader set available without app-provided custom shader definitions

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- checker does not need to be part of the built-in shader set
- the initial proving shader is weight-sin
