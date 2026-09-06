# thaum-renderer/domain/modules/individuals/ui-customization

## purpose
Own a generic UI-color customization module: one renderer-owned floating panel listing the semantic UI roles and letting a consumer apply its current left-hand or right-hand color directly onto any role with one click.

## owns
- the `UiCustomizationModule` type and its `Module` implementation
- drawing one row per `UiColorRole` with the role label and current role swatch
- left-click applying the consumer's current left color to the clicked role
- right-click applying the consumer's current right color to the clicked role
- gizmo-bar-backed move/close/resize/seamless behavior

## does not own
- where the consumer's left/right colors come from
- disk persistence policy for customized UI palettes
- app-specific recall/menu wiring in consuming programs

## children-encapsulations
- none

## contents
- `ui_customization_module.rs`
  - `UiCustomizationModule` and its inline tests

## dependencies
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/modules/shared/`

## exposed interfaces
- `UiCustomizationModule::new(id, rect, palette, get_left_rgb, get_right_rgb)`

## interface consumers
- `thaum-painter/orchestration/entrypoint/`
- future renderer consumers wanting the same semantic UI-color editor

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `ui_customization_module.rs`
  - light
  - validates left/right click role assignment and helper-row miss behavior

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/modules/customization_module.ts`, but this version removes the extra spawned color-picker step and applies the already-selected left/right color directly, per J's request.
