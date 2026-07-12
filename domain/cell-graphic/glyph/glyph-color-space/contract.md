# /home/j/Repos/thaum-renderer/domain/cell-graphic/glyph/glyph-color-space

## purpose
Own the glyph-specific source color-space contract while resolving into the same renderer color/material output shape used by sprites.

## owns
- the canonical glyph color-space contract
- binary glyph source decode rules
- the rule that black resolves as transparent in glyph binary mode
- the rule that white resolves through the medium-light band by default
- the rule that material-backed glyphs use the medium-light band in the current renderer shape
- support for mono or grayscale source assets that still collapse into the shared renderer resolution shape
- glyph-side compatibility with the same flat-color and material resolution seam used by sprites

## does not own
- sprite color-space rules
- material library ownership
- shader stack ownership
- app-specific text semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/glyph/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-materials/`

## exposed interfaces
- glyph color-space shape
  - describes how glyph or mono source pixels map into the shared renderer channel and band resolution seam

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/glyph/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the current medium-light default is an intentional temporary assumption and may change later without forking the wider renderer color system
