# thaum-renderer/orchestration/asset-root

## purpose
Own the boot-time asset-root seam that lets a consuming program point thaum-renderer at one chosen renderer-assets location without forcing broader app folder organization.

## owns
- the consumer-provided renderer asset-root contract
- the rule that the consuming app chooses one root location for renderer-owned intake
- boot-time expectations for renderer-facing subfolders such as glyph fonts, cell sprites, materials, cell shaders, cell effects, and post effects
- the stable place hot reload may watch when enabled
- renderer-owned documentation of the external folder shape expected from consumer assets

## does not own
- app-wide project layout outside the chosen renderer root
- renderer runtime drawing behavior
- consumer-authored asset contents themselves
- ownership of where a consumer keeps its repo, app code, or launch code

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/orchestration/`

## exposed interfaces
- asset-root shape
  - describes the single boot-provided root path the renderer uses to resolve expected renderer asset folders

## interface consumers
- `thaum-renderer/orchestration/boot/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- the renderer should expect one consumer-chosen root path rather than forcing a full app repo shape
- the consumer app may keep that root inside its own repo, beside its app code, or elsewhere it chooses
- the first proving folder shape under that root should include `glyph-fonts/` and `cell-sprites/`
- sibling folders should be expected for `materials/`, `cell-shaders/`, `cell-effects/`, and `post-effects/` even if some are empty early on
- hot reload should watch this root when enabled
- renderer-owned proving assets should not be required to live inside the renderer repo
