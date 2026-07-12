# /home/j/Repos/thaum-renderer/domain/camera/screen-world-remap

## purpose
Own the camera-facing remap semantics between screen space and world space.

## owns
- screen-to-world and world-to-screen remap truth that belongs to camera semantics
- the mapping seam used by renderer viewing and camera-relative interactions

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
