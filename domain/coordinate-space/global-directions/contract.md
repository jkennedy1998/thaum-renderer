# thaum-renderer/domain/coordinate-space/global-directions

## purpose
Own the renderer-global direction vocabulary that maps cardinal direction language onto the shared world xyz coordinate space.

## owns
- canonical global direction naming like top, bottom, north, east, south, and west
- the mapping between renderer world axes and global direction language
- six-cardinal direction truth consumed by camera-facing and group-facing systems
- renderer-owned facing rules that depend on stable global direction language

## does not own
- camera-relative view direction or active depth selection
- group-local facing state
- app-specific semantic region naming

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- global direction shape
  - describes the stable cardinal direction vocabulary and axis mapping used across renderer world space

## interface consumers
- `thaum-renderer/domain/coordinate-space/`
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/cell-group/transform/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this seam owns global direction words, not camera-relative viewing words
- camera may derive an active viewing plane from these directions, but should not become the source of truth for the directions themselves
- this keeps global facing rules stable even when camera swing or roll changes the current viewing relationship
- north/east/south/west/top/bottom naming should stay reusable across camera and cell-group contracts so view-family wording and global-facing wording can stay interchangeable where appropriate
