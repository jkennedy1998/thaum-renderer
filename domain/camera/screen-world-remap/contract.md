# /home/j/Repos/thaum-renderer/domain/camera/screen-world-remap

## purpose
Own the camera-facing remap semantics between screen space and world space.

## owns
- screen-to-world and world-to-screen remap truth that belongs to camera semantics
- the mapping seam used by renderer viewing and camera-relative interactions
- remap between screen-space positions and the camera-derived focus plane or requested visible plane
- agreement rules between remap behavior and camera projection authority
- the alignment rule that 2d focus-plane content and 3d focus-plane content at the same global point must resolve to the same screen place

## does not own
- app-specific input logic
- generic matrix helper utilities
- cell-group composition ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/camera/`
- `/home/j/Repos/thaum-renderer/domain/camera/projection/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- screen-world remap shape
  - describes how camera semantics translate between viewed screen-space and renderer world-space references

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/camera/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this seam keeps remap semantics explicit rather than burying them inside projection details
- program-facing screen/world conversion should come from this renderer seam rather than from ad hoc cursor logic in camera or composition
- remap behavior should agree with the projection-owned plane stack and the camera-owned focus target depth anchor
- renderer remap should be usable by downstream programs in both directions so program-owned cursors can convert screen/window coordinates to cells and cells back to screen/window coordinates
- the older "active plane" wording is too narrow if it implies cursor ownership; the core truth here is focus-plane and plane-requested remap under shared camera perspective
