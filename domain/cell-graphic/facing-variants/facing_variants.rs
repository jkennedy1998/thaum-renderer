use std::collections::BTreeMap;

use crate::cell_facing::{CellFacing, FacingRotation};
use crate::cell_graphic::SpriteGraphic;

/// One side's graphic: the flat art a Sided object shows from one orientation.
/// Never a nested Sided — a side is a flat look, recursion would be
/// meaningless and would keep the type from staying simple.
///
/// A side declaration is graphic-only: color and weight are cell-level
/// concerns (the cell's color slots), not per-side declaration data, so the
/// resolved side replaces only the cell's graphic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SideGraphic {
    None,
    Glyph(char),
    Sprite(SpriteGraphic),
}

/// One exact orientation's (facing x roll) declared side: its own graphic or
/// a same-as alias to another orientation's entry. Roll-specific sides are
/// expressible here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrientationVariant {
    Look(SideGraphic),
    SameAs(FacingRotation),
}

/// One facing's roll-agnostic side: applies at every roll of that facing that
/// has no exact orientation entry. Either its own graphic or a same-as alias
/// to another facing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FacingVariant {
    Look(SideGraphic),
    SameAs(CellFacing),
}

/// The Sided graphic kind: one declaration that maps orientations onto side
/// graphics, so a six-sided 3D cell can rotate using its cell facing. The
/// key space is the 24-element orientation group — declare the 6 sides, add
/// side+roll entries only where custom roll art exists (J 2026-09-12).
///
/// Declared where sprites are declared; consuming programs declare their own
/// Sided objects. When the ladder lands on no declared entry, resolution
/// falls through to the cell's own authored appearance (`None` here) unless
/// the declaration carries its own base graphic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SidedGraphic {
    base: Option<SideGraphic>,
    orientation_variants: BTreeMap<FacingRotation, OrientationVariant>,
    facing_variants: BTreeMap<CellFacing, FacingVariant>,
}

impl SidedGraphic {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a base graphic: the graphic undeclared sides resolve to.
    /// Without one, undeclared sides fall through to the cell's own
    /// appearance.
    pub fn with_base(mut self, base: SideGraphic) -> Self {
        self.base = Some(base);
        self
    }

    /// Declare one exact relative orientation's side (facing + roll).
    pub fn with_orientation_variant(
        mut self,
        orientation: FacingRotation,
        variant: OrientationVariant,
    ) -> Self {
        self.orientation_variants.insert(orientation, variant);
        self
    }

    /// Declare one facing's roll-agnostic side: resolves at every roll of
    /// that facing that has no exact orientation entry.
    pub fn with_facing_variant(mut self, facing: CellFacing, variant: FacingVariant) -> Self {
        self.facing_variants.insert(facing, variant);
        self
    }

    pub fn base(&self) -> Option<&SideGraphic> {
        self.base.as_ref()
    }

    pub fn orientation_variant(&self, orientation: FacingRotation) -> Option<&OrientationVariant> {
        self.orientation_variants.get(&orientation)
    }

    pub fn facing_variant(&self, facing: CellFacing) -> Option<&FacingVariant> {
        self.facing_variants.get(&facing)
    }

    /// All declared facing-tier entries, canonical facing order.
    pub fn facing_variant_map(&self) -> &BTreeMap<CellFacing, FacingVariant> {
        &self.facing_variants
    }

    /// All declared orientation-tier entries, canonical key order.
    pub fn orientation_variant_map(&self) -> &BTreeMap<FacingRotation, OrientationVariant> {
        &self.orientation_variants
    }

    /// Resolve the effective side graphic for one composed relative
    /// orientation by the fallback ladder: the exact orientation entry first,
    /// then the roll-agnostic facing entry, then the declared base. `None`
    /// means no side was declared here — the consumer falls through to the
    /// cell's own authored appearance. Same-as chains are followed within
    /// each tier up to the tier's size; a cycle is a broken asset and
    /// degrades to the next tier.
    pub fn resolve(&self, relative: FacingRotation) -> Option<&SideGraphic> {
        let mut current = relative;
        for _ in 0..24 {
            match self.orientation_variants.get(&current) {
                Some(OrientationVariant::Look(graphic)) => return Some(graphic),
                Some(OrientationVariant::SameAs(next)) => current = *next,
                None => return self.resolve_facing(current.facing),
            }
        }
        self.resolve_facing(current.facing)
    }

    fn resolve_facing(&self, facing: CellFacing) -> Option<&SideGraphic> {
        let mut current = facing;
        for _ in 0..6 {
            match self.facing_variants.get(&current) {
                Some(FacingVariant::Look(graphic)) => return Some(graphic),
                Some(FacingVariant::SameAs(next)) => current = *next,
                None => return self.base.as_ref(),
            }
        }
        self.base.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_facing::{CellFacing, CellRoll};

    fn sprite(name: &str) -> SideGraphic {
        SideGraphic::Sprite(SpriteGraphic::new(format!("proofs/{name}.png")))
    }

    #[test]
    fn no_entries_and_no_base_resolve_to_none_from_every_direction() {
        let sided = SidedGraphic::new();
        for &facing in &CellFacing::ALL {
            assert_eq!(
                sided.resolve(FacingRotation::from_facing(facing)),
                None,
                "undeclared sides fall through to the cell's own look"
            );
        }
    }

    #[test]
    fn a_declared_base_resolves_from_undeclared_directions() {
        let sided = SidedGraphic::new().with_base(sprite("chest"));
        for &facing in &CellFacing::ALL {
            assert_eq!(
                sided.resolve(FacingRotation::from_facing(facing)),
                Some(&sprite("chest"))
            );
        }
    }

    #[test]
    fn side_entries_carry_a_graphic_per_side() {
        let front = SideGraphic::Glyph('R');
        let east = sprite("chest-east");
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosZ, FacingVariant::Look(front))
            .with_facing_variant(CellFacing::PosX, FacingVariant::Look(east));

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosZ)),
            Some(&SideGraphic::Glyph('R'))
        );
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            Some(&sprite("chest-east"))
        );
    }

    #[test]
    fn orientation_side_entries_resolve_by_relative_facing() {
        let sided = SidedGraphic::new()
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::PosX),
                OrientationVariant::Look(sprite("chest-east")),
            )
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::NegX),
                OrientationVariant::Look(sprite("chest-west")),
            );

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            Some(&sprite("chest-east"))
        );
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::NegX)),
            Some(&sprite("chest-west"))
        );
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosZ)),
            None
        );
    }

    #[test]
    fn same_as_chains_resolve_to_the_target_graphic() {
        let sided = SidedGraphic::new()
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::PosX),
                OrientationVariant::Look(sprite("sign-east")),
            )
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::NegX),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::PosX)),
            );

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::NegX)),
            Some(&sprite("sign-east"))
        );
    }

    #[test]
    fn facing_sides_resolve_at_every_roll_of_that_facing() {
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosZ, FacingVariant::Look(sprite("tile-top")));

        for &roll in &CellRoll::ALL {
            assert_eq!(
                sided.resolve(FacingRotation::new(CellFacing::PosZ, roll)),
                Some(&sprite("tile-top"))
            );
        }
        // Other facings fall through: no base declared.
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            None
        );
    }

    #[test]
    fn exact_orientation_entry_outranks_the_facing_entry() {
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosZ, FacingVariant::Look(sprite("tile-top")))
            .with_orientation_variant(
                FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90),
                OrientationVariant::Look(sprite("tile-top-tilted")),
            );

        assert_eq!(
            sided.resolve(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90)),
            Some(&sprite("tile-top-tilted"))
        );
        assert_eq!(
            sided.resolve(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg180)),
            Some(&sprite("tile-top"))
        );
    }

    #[test]
    fn facing_same_as_resolves_to_the_target_graphic() {
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosZ, FacingVariant::Look(sprite("sign-top")))
            .with_facing_variant(CellFacing::NegZ, FacingVariant::SameAs(CellFacing::PosZ));

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::NegZ)),
            Some(&sprite("sign-top"))
        );
    }

    #[test]
    fn exact_chain_landing_on_an_unlisted_orientation_degrades_to_its_facing_tier() {
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosZ, FacingVariant::Look(sprite("dial-front")))
            .with_orientation_variant(
                FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90),
                OrientationVariant::SameAs(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg180)),
            );

        // The Deg90 entry aliases Deg180, which has no exact entry, so the
        // chain degrades to PosZ's facing tier instead of the base.
        assert_eq!(
            sided.resolve(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90)),
            Some(&sprite("dial-front"))
        );
    }

    #[test]
    fn roll_specific_sides_resolve_separately_from_unrolled_views() {
        let front = FacingRotation::from_facing(CellFacing::PosZ);
        let front_rolled = FacingRotation::new(CellFacing::PosZ, CellRoll::Deg90);
        let sided = SidedGraphic::new()
            .with_orientation_variant(front, OrientationVariant::Look(sprite("dial-front")))
            .with_orientation_variant(
                front_rolled,
                OrientationVariant::Look(sprite("dial-tilted")),
            );

        assert_eq!(sided.resolve(front), Some(&sprite("dial-front")));
        assert_eq!(sided.resolve(front_rolled), Some(&sprite("dial-tilted")));
    }

    #[test]
    fn same_as_cycles_fall_through() {
        let sided = SidedGraphic::new()
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::PosX),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::NegX)),
            )
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::NegX),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::PosX)),
            );

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            None
        );
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::NegX)),
            None
        );
    }

    #[test]
    fn facing_same_as_cycles_fall_through() {
        let sided = SidedGraphic::new()
            .with_facing_variant(CellFacing::PosX, FacingVariant::SameAs(CellFacing::NegX))
            .with_facing_variant(CellFacing::NegX, FacingVariant::SameAs(CellFacing::PosX));

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            None
        );
        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::NegX)),
            None
        );
    }

    #[test]
    fn same_as_cycles_degrade_to_a_declared_base() {
        let sided = SidedGraphic::new()
            .with_base(sprite("broken"))
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::PosX),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::NegX)),
            )
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::NegX),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::PosX)),
            );

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosX)),
            Some(&sprite("broken"))
        );
    }

    #[test]
    fn self_referential_same_as_falls_through_to_the_base() {
        let sided = SidedGraphic::new()
            .with_base(sprite("loop"))
            .with_orientation_variant(
                FacingRotation::from_facing(CellFacing::PosY),
                OrientationVariant::SameAs(FacingRotation::from_facing(CellFacing::PosY)),
            );

        assert_eq!(
            sided.resolve(FacingRotation::from_facing(CellFacing::PosY)),
            Some(&sprite("loop"))
        );
    }
}
