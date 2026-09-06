# thaum-renderer/domain/cell-graphic/sprite/sprite-color-space

## purpose
Own the renderer sprite color-space contract used to decode sprite pixels into shared renderer color/material resolution inputs.

## owns
- the canonical sprite color-space contract
- sprite-defined color-space rules rather than shader-owned color decoding
- six supported sprite channels: `A`, `B`, `C`, `A+B`, `B+C`, and `C+A`
- four fixed value bands: darkest, medium-dark, medium-light, and lightest
- the rule that blend channels resolve as 50% mixes of the paired channel outputs at the same band
- nearest-color matching against the declared sprite color-space with no warning path
- support for both load-time and runtime/post-process nearest-color matching use
- canonical shared decode expectations for sprite source assets provided as pngs

## does not own
- sprite atlas stacking or packed sprite layout
- material library ownership
- shader stack ownership
- app-specific light semantics

## children-encapsulations
- none

## contents
- `sprite_color_space.rs`
  - rust sprite color-space decode and canonical palette helpers owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell-graphic/sprite/`
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-materials/`
- `thaum-renderer/tools/color/`

## exposed interfaces
- sprite color-space shape
  - describes how renderer sprite pixels map into channel identity and value-band identity before flat-color or material resolution
- canonical sprite palette declaration
  - describes the per-sprite declared color collection used for nearest-color matching and decode

## interface consumers
- `thaum-renderer/domain/cell-graphic/sprite/`
- `thaum-renderer/domain/cell-color/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the first known sprite palette shape is twenty four colors covering six channels across four value bands
- sprites define their own color-space declaration inside this renderer-owned decode format
- color-space decode is renderer-wide behavior and is not limited to cell-shader use
