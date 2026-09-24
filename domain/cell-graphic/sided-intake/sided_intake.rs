//! Portable Sided declaration intake: parse one declaration file into a
//! runtime [`SidedGraphic`]. The declaration format is the portable asset
//! shape consuming programs author by hand or tool; this boundary owns the
//! format vocabulary (kebab keys, tiers, same-as aliasing) and its
//! validation, so every consumer shares one parser and one error surface.
//!
//! Declarations are graphic-only: a side names which art it shows and
//! nothing else. Color resolves through the cell's own color slots and
//! weight belongs to gameplay state, so neither appears in the format.

use std::fmt;
use std::path::Path;

use serde_json::Value;

use crate::cell_facing::{CellFacing, CellRoll, FacingRotation};
use crate::cell_graphic::facing_variants::{
    FacingVariant, OrientationVariant, SideGraphic, SidedGraphic,
};
use crate::cell_graphic::SpriteGraphic;

/// Format tag every declaration file carries.
pub const SIDED_DECLARATION_FORMAT_TAG: &str = "thaum-sided-declaration";
/// The declaration format version this parser reads.
pub const SIDED_DECLARATION_VERSION: u32 = 1;
/// Portable facing key vocabulary, kebab-case.
pub const SIDED_DECLARATION_FACING_VOCABULARY: &str = "pos-x, neg-x, pos-y, neg-y, pos-z, neg-z";
/// Portable roll key vocabulary, kebab-case. `deg-90` is 90° counter-clockwise
/// viewed from the canonical front: an upright stick tips to the left.
/// `deg-270` tips to the right.
pub const SIDED_DECLARATION_ROLL_VOCABULARY: &str = "deg-0, deg-90, deg-180, deg-270";

/// One parsed declaration failure, with an author-facing message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidedDeclarationError(pub String);

impl fmt::Display for SidedDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sided declaration: {}", self.0)
    }
}

impl std::error::Error for SidedDeclarationError {}

pub fn facing_portable_name(facing: CellFacing) -> &'static str {
    match facing {
        CellFacing::PosX => "pos-x",
        CellFacing::NegX => "neg-x",
        CellFacing::PosY => "pos-y",
        CellFacing::NegY => "neg-y",
        CellFacing::PosZ => "pos-z",
        CellFacing::NegZ => "neg-z",
    }
}

pub fn facing_from_portable_name(name: &str) -> Option<CellFacing> {
    match name {
        "pos-x" => Some(CellFacing::PosX),
        "neg-x" => Some(CellFacing::NegX),
        "pos-y" => Some(CellFacing::PosY),
        "neg-y" => Some(CellFacing::NegY),
        "pos-z" => Some(CellFacing::PosZ),
        "neg-z" => Some(CellFacing::NegZ),
        _ => None,
    }
}

pub fn roll_portable_name(roll: CellRoll) -> &'static str {
    match roll {
        CellRoll::Deg0 => "deg-0",
        CellRoll::Deg90 => "deg-90",
        CellRoll::Deg180 => "deg-180",
        CellRoll::Deg270 => "deg-270",
    }
}

pub fn roll_from_portable_name(name: &str) -> Option<CellRoll> {
    match name {
        "deg-0" => Some(CellRoll::Deg0),
        "deg-90" => Some(CellRoll::Deg90),
        "deg-180" => Some(CellRoll::Deg180),
        "deg-270" => Some(CellRoll::Deg270),
        _ => None,
    }
}

/// The portable orientation key for one exact side+roll entry:
/// `<facing>/<roll>`, e.g. `pos-z/deg-90`.
pub fn orientation_portable_key(orientation: FacingRotation) -> String {
    format!(
        "{}/{}",
        facing_portable_name(orientation.facing),
        roll_portable_name(orientation.roll)
    )
}

fn orientation_from_portable_key(key: &str) -> Option<FacingRotation> {
    let (facing, roll) = key.split_once('/')?;
    Some(FacingRotation::new(
        facing_from_portable_name(facing)?,
        roll_from_portable_name(roll)?,
    ))
}

/// Parse one declaration file's text into a runtime [`SidedGraphic`].
pub fn parse_sided_declaration(json: &str) -> Result<SidedGraphic, SidedDeclarationError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| SidedDeclarationError(format!("invalid json: {e}")))?;
    sided_declaration_from_value(&value)
}

/// Load and parse one declaration file from disk.
pub fn load_sided_declaration(path: &Path) -> Result<SidedGraphic, SidedDeclarationError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| SidedDeclarationError(format!("cannot read {}: {e}", path.display())))?;
    parse_sided_declaration(&text)
}

fn sided_declaration_from_value(value: &Value) -> Result<SidedGraphic, SidedDeclarationError> {
    let object = value
        .as_object()
        .ok_or_else(|| SidedDeclarationError("top level must be a json object".to_owned()))?;

    match object.get("format") {
        Some(Value::String(tag)) if tag == SIDED_DECLARATION_FORMAT_TAG => {}
        Some(Value::String(tag)) => {
            return Err(SidedDeclarationError(format!(
                "unknown format tag \"{tag}\", expected \"{SIDED_DECLARATION_FORMAT_TAG}\""
            )))
        }
        _ => {
            return Err(SidedDeclarationError(format!(
                "missing \"format\": \"{SIDED_DECLARATION_FORMAT_TAG}\""
            )))
        }
    }

    match object.get("version") {
        Some(Value::Number(n)) if n.as_u64() == Some(SIDED_DECLARATION_VERSION as u64) => {}
        _ => {
            return Err(SidedDeclarationError(format!(
                "missing or unsupported \"version\": expected {SIDED_DECLARATION_VERSION}"
            )))
        }
    }

    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "format" | "version" | "base" | "sides" | "orientations"
        ) {
            return Err(SidedDeclarationError(format!(
                "unknown top-level key \"{key}\" (expected format, version, base, sides, orientations)"
            )));
        }
    }

    let base = match object.get("base") {
        Some(value) => Some(portable_graphic(value, "base")?),
        None => None,
    };

    let mut sided = SidedGraphic::new();
    if let Some(base) = base {
        sided = sided.with_base(base);
    }

    // Facing tier: roll-agnostic sides.
    let sides = object.get("sides");
    let side_keys: Vec<(CellFacing, PortableEntry)> = match sides {
        Some(Value::Object(map)) => {
            let mut entries = Vec::new();
            for (key, value) in map {
                let facing = facing_from_portable_name(key).ok_or_else(|| {
                    SidedDeclarationError(format!(
                        "sides: unknown facing key \"{key}\" (vocabulary: {SIDED_DECLARATION_FACING_VOCABULARY})"
                    ))
                })?;
                entries.push((facing, portable_entry(value, &format!("sides.{key}"))?));
            }
            entries
        }
        Some(_) => {
            return Err(SidedDeclarationError(
                "\"sides\" must be an object".to_owned(),
            ))
        }
        None => Vec::new(),
    };

    // Orientation tier: exact side+roll entries.
    let orientations = object.get("orientations");
    let orientation_keys: Vec<(FacingRotation, PortableEntry)> = match orientations {
        Some(Value::Object(map)) => {
            let mut entries = Vec::new();
            for (key, value) in map {
                let orientation = orientation_from_portable_key(key).ok_or_else(|| {
                    SidedDeclarationError(format!(
                        "orientations: unknown key \"{key}\" (expected <facing>/<roll>; vocabularies: {SIDED_DECLARATION_FACING_VOCABULARY} / {SIDED_DECLARATION_ROLL_VOCABULARY})"
                    ))
                })?;
                entries.push((
                    orientation,
                    portable_entry(value, &format!("orientations.{key}"))?,
                ));
            }
            entries
        }
        Some(_) => {
            return Err(SidedDeclarationError(
                "\"orientations\" must be an object".to_owned(),
            ))
        }
        None => Vec::new(),
    };

    // Same-as targets must exist within their own tier, and chains must be
    // cycle-free: a broken alias is a broken asset, so intake rejects it with
    // a precise message instead of degrading silently at render time.
    for (facing, entry) in &side_keys {
        if let PortableEntry::SameAs(target) = entry {
            validate_side_alias(*facing, target, &side_keys)?;
        }
    }
    for (orientation, entry) in &orientation_keys {
        if let PortableEntry::SameAs(target) = entry {
            validate_orientation_alias(*orientation, target, &orientation_keys)?;
        }
    }

    for (facing, entry) in side_keys {
        let variant = match entry {
            PortableEntry::Graphic(graphic) => FacingVariant::Look(graphic),
            PortableEntry::SameAs(target) => {
                FacingVariant::SameAs(facing_from_portable_name(&target).ok_or_else(|| {
                    SidedDeclarationError(format!(
                        "sides: same-as target \"{target}\" is not a facing key"
                    ))
                })?)
            }
        };
        sided = sided.with_facing_variant(facing, variant);
    }
    for (orientation, entry) in orientation_keys {
        let variant = match entry {
            PortableEntry::Graphic(graphic) => OrientationVariant::Look(graphic),
            PortableEntry::SameAs(target) => {
                OrientationVariant::SameAs(
                    orientation_from_portable_key(&target).ok_or_else(|| {
                        SidedDeclarationError(format!(
                            "orientations: same-as target \"{target}\" is not an orientation key (<facing>/<roll>)"
                        ))
                    })?,
                )
            }
        };
        sided = sided.with_orientation_variant(orientation, variant);
    }

    Ok(sided)
}

enum PortableEntry {
    Graphic(SideGraphic),
    SameAs(String),
}

fn portable_entry(value: &Value, at: &str) -> Result<PortableEntry, SidedDeclarationError> {
    if let Some(target) = value.get("same-as") {
        let target = target.as_str().ok_or_else(|| {
            SidedDeclarationError(format!("{at}: \"same-as\" must be a string key"))
        })?;
        if target.is_empty() {
            return Err(SidedDeclarationError(format!(
                "{at}: \"same-as\" must name a declared key in this tier"
            )));
        }
        return Ok(PortableEntry::SameAs(target.to_owned()));
    }
    Ok(PortableEntry::Graphic(portable_graphic(value, at)?))
}

fn validate_side_alias(
    from: CellFacing,
    target_name: &str,
    side_keys: &[(CellFacing, PortableEntry)],
) -> Result<(), SidedDeclarationError> {
    let target = facing_from_portable_name(target_name).ok_or_else(|| {
        SidedDeclarationError(format!(
            "sides: same-as target \"{target_name}\" is not a facing key"
        ))
    })?;
    let mut current = target;
    for _ in 0..=side_keys.len() {
        if current == from {
            return Err(SidedDeclarationError(format!(
                "sides: same-as chain from \"{}\" cycles through itself",
                facing_portable_name(from)
            )));
        }
        let Some((_, entry)) = side_keys.iter().find(|(facing, _)| *facing == current) else {
            return Err(SidedDeclarationError(format!(
                "sides: same-as target \"{}\" is not declared in this tier",
                target_name
            )));
        };
        match entry {
            PortableEntry::Graphic(_) => return Ok(()),
            PortableEntry::SameAs(next) => {
                current = facing_from_portable_name(next).ok_or_else(|| {
                    SidedDeclarationError(format!(
                        "sides: same-as chain reaches unknown key \"{next}\""
                    ))
                })?;
            }
        }
    }
    Err(SidedDeclarationError(format!(
        "sides: same-as chain from \"{}\" is too long to resolve",
        facing_portable_name(from)
    )))
}

fn validate_orientation_alias(
    from: FacingRotation,
    target_name: &str,
    orientation_keys: &[(FacingRotation, PortableEntry)],
) -> Result<(), SidedDeclarationError> {
    let target = orientation_from_portable_key(target_name).ok_or_else(|| {
        SidedDeclarationError(format!(
            "orientations: same-as target \"{target_name}\" is not an orientation key (<facing>/<roll>)"
        ))
    })?;
    let mut current = target;
    for _ in 0..=orientation_keys.len() {
        if current == from {
            return Err(SidedDeclarationError(format!(
                "orientations: same-as chain from \"{}\" cycles through itself",
                orientation_portable_key(from)
            )));
        }
        let Some((_, entry)) = orientation_keys
            .iter()
            .find(|(orientation, _)| *orientation == current)
        else {
            return Err(SidedDeclarationError(format!(
                "orientations: same-as target \"{}\" is not declared in this tier",
                target_name
            )));
        };
        match entry {
            PortableEntry::Graphic(_) => return Ok(()),
            PortableEntry::SameAs(next) => {
                current = orientation_from_portable_key(next).ok_or_else(|| {
                    SidedDeclarationError(format!(
                        "orientations: same-as chain reaches unknown key \"{next}\""
                    ))
                })?;
            }
        }
    }
    Err(SidedDeclarationError(format!(
        "orientations: same-as chain from \"{}\" is too long to resolve",
        orientation_portable_key(from)
    )))
}

/// Parse one side entry's graphic. Entries are graphic-only: an object with
/// exactly one of `glyph` or `sprite`, or the string `"none"` for a side
/// that deliberately shows nothing. Anything else — including the retired
/// per-side `color`, `weight`, or `shader-stack` keys, which moved to the
/// cell level — is rejected with a pointing error.
fn portable_graphic(value: &Value, at: &str) -> Result<SideGraphic, SidedDeclarationError> {
    match value {
        // "none": the side deliberately shows nothing.
        Value::String(name) if name == "none" => Ok(SideGraphic::None),
        Value::String(other) => Err(SidedDeclarationError(format!(
            "{at}: unknown graphic \"{other}\" (expected an object or \"none\")"
        ))),
        Value::Object(map) => {
            if map.contains_key("color")
                || map.contains_key("weight")
                || map.contains_key("shader-stack")
            {
                return Err(SidedDeclarationError(format!(
                    "{at}: side entries are graphic-only; \"color\", \"weight\", and \"shader-stack\" live on the cell, not in the declaration"
                )));
            }
            if let Some(Value::String(glyph)) = map.get("glyph") {
                let mut chars = glyph.chars();
                let glyph_char = chars.next().ok_or_else(|| {
                    SidedDeclarationError(format!("{at}: \"glyph\" must be one character"))
                })?;
                if chars.next().is_some() {
                    return Err(SidedDeclarationError(format!(
                        "{at}: \"glyph\" must be exactly one character"
                    )));
                }
                if map.len() != 1 {
                    return Err(SidedDeclarationError(format!(
                        "{at}: graphic object must declare exactly one of glyph, sprite"
                    )));
                }
                return Ok(SideGraphic::Glyph(glyph_char));
            }
            if let Some(Value::String(sprite)) = map.get("sprite") {
                if sprite.is_empty() {
                    return Err(SidedDeclarationError(format!(
                        "{at}: \"sprite\" must be an atlas-relative path"
                    )));
                }
                if map.len() != 1 {
                    return Err(SidedDeclarationError(format!(
                        "{at}: graphic object must declare exactly one of glyph, sprite"
                    )));
                }
                return Ok(SideGraphic::Sprite(SpriteGraphic::new(sprite)));
            }
            Err(SidedDeclarationError(format!(
                "{at}: graphic object must declare exactly one of glyph, sprite (or be the string \"none\")"
            )))
        }
        _ => Err(SidedDeclarationError(format!(
            "{at}: graphic must be an object or the string \"none\""
        ))),
    }
}

/// Serialize one runtime `SidedGraphic` back into canonical declaration
/// text, so persistence and clipboard payloads can carry a declaration
/// inline without depending on the originating file. Round-trips through
/// [`parse_sided_declaration`].
pub fn sided_declaration_json(sided: &SidedGraphic) -> String {
    let mut object = serde_json::Map::new();
    object.insert(
        "format".to_owned(),
        Value::String(SIDED_DECLARATION_FORMAT_TAG.to_owned()),
    );
    object.insert(
        "version".to_owned(),
        Value::Number(SIDED_DECLARATION_VERSION.into()),
    );
    if let Some(base) = sided.base() {
        object.insert("base".to_owned(), graphic_value(base));
    }
    if !sided.facing_variant_map().is_empty() {
        let mut sides = serde_json::Map::new();
        for (facing, variant) in sided.facing_variant_map() {
            sides.insert(
                facing_portable_name(*facing).to_owned(),
                match variant {
                    FacingVariant::Look(graphic) => graphic_value(graphic),
                    FacingVariant::SameAs(target) => same_as_value(facing_portable_name(*target)),
                },
            );
        }
        object.insert("sides".to_owned(), Value::Object(sides));
    }
    if !sided.orientation_variant_map().is_empty() {
        let mut orientations = serde_json::Map::new();
        for (rotation, variant) in sided.orientation_variant_map() {
            orientations.insert(
                orientation_portable_key(*rotation),
                match variant {
                    OrientationVariant::Look(graphic) => graphic_value(graphic),
                    OrientationVariant::SameAs(target) => {
                        same_as_value(&orientation_portable_key(*target))
                    }
                },
            );
        }
        object.insert("orientations".to_owned(), Value::Object(orientations));
    }
    serde_json::to_string_pretty(&Value::Object(object)).unwrap_or_else(|_| "{}".to_owned())
}

fn same_as_value(target: &str) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("same-as".to_owned(), Value::String(target.to_owned()));
    Value::Object(entry)
}

fn graphic_value(graphic: &SideGraphic) -> Value {
    match graphic {
        SideGraphic::None => Value::String("none".to_owned()),
        SideGraphic::Glyph(glyph) => {
            let mut object = serde_json::Map::new();
            object.insert("glyph".to_owned(), Value::String(glyph.to_string()));
            Value::Object(object)
        }
        SideGraphic::Sprite(sprite) => {
            let mut object = serde_json::Map::new();
            object.insert(
                "sprite".to_owned(),
                Value::String(sprite.atlas_relative_path().to_string_lossy().into_owned()),
            );
            Value::Object(object)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The canonical sample: a stick. Upright horizontals show `┃` (declared
    /// non-rolled, so every roll inherits), the stick lying over shows `━`
    /// on the horizontals (side+roll entries), and the ends show `▪` at
    /// every roll. Graphic-only: color and weight live on the cell, not here.
    const STICK_JSON: &str = r#"
    {
      "format": "thaum-sided-declaration",
      "version": 1,
      "sides": {
        "pos-z": { "glyph": "┃" },
        "neg-z": { "same-as": "pos-z" },
        "pos-x": { "same-as": "pos-z" },
        "neg-x": { "same-as": "pos-z" },
        "pos-y": { "glyph": "▪" },
        "neg-y": { "same-as": "pos-y" }
      },
      "orientations": {
        "pos-z/deg-90": { "glyph": "━" },
        "neg-z/deg-90": { "same-as": "pos-z/deg-90" },
        "pos-x/deg-90": { "same-as": "pos-z/deg-90" },
        "neg-x/deg-90": { "same-as": "pos-z/deg-90" },
        "pos-z/deg-270": { "same-as": "pos-z/deg-90" },
        "neg-z/deg-270": { "same-as": "pos-z/deg-90" },
        "pos-x/deg-270": { "same-as": "pos-z/deg-90" },
        "neg-x/deg-270": { "same-as": "pos-z/deg-90" }
      }
    }"#;

    fn stick() -> SidedGraphic {
        parse_sided_declaration(STICK_JSON).expect("stick declaration parses")
    }

    fn resolved_glyph(sided: &SidedGraphic, facing: CellFacing, roll: CellRoll) -> char {
        match &sided
            .resolve(FacingRotation::new(facing, roll))
            .expect("side declared")
        {
            SideGraphic::Glyph(glyph) => *glyph,
            other => panic!("expected glyph, got {other:?}"),
        }
    }

    #[test]
    fn stick_horizontals_show_the_upright_profile_at_every_non_overridden_roll() {
        let sided = stick();
        for facing in [
            CellFacing::PosX,
            CellFacing::NegX,
            CellFacing::PosZ,
            CellFacing::NegZ,
        ] {
            assert_eq!(resolved_glyph(&sided, facing, CellRoll::Deg0), '┃');
            assert_eq!(resolved_glyph(&sided, facing, CellRoll::Deg180), '┃');
        }
    }

    #[test]
    fn stick_horizontals_show_the_lying_profile_when_rolled_a_quarter_turn() {
        let sided = stick();
        for facing in [
            CellFacing::PosX,
            CellFacing::NegX,
            CellFacing::PosZ,
            CellFacing::NegZ,
        ] {
            assert_eq!(resolved_glyph(&sided, facing, CellRoll::Deg90), '━');
            assert_eq!(resolved_glyph(&sided, facing, CellRoll::Deg270), '━');
        }
    }

    #[test]
    fn stick_ends_show_the_end_grain_at_every_roll() {
        let sided = stick();
        for facing in [CellFacing::PosY, CellFacing::NegY] {
            for roll in CellRoll::ALL {
                assert_eq!(resolved_glyph(&sided, facing, roll), '▪');
            }
        }
    }

    #[test]
    fn base_graphic_resolves_where_no_side_is_declared() {
        let sided = parse_sided_declaration(
            r#"{
                "format": "thaum-sided-declaration",
                "version": 1,
                "base": { "glyph": "?" },
                "sides": { "pos-z": { "glyph": "f" } }
            }"#,
        )
        .unwrap();
        assert_eq!(
            resolved_glyph(&sided, CellFacing::NegX, CellRoll::Deg0),
            '?'
        );
        assert_eq!(
            resolved_glyph(&sided, CellFacing::PosZ, CellRoll::Deg0),
            'f'
        );
    }

    #[test]
    fn undeclared_everything_falls_through_as_none() {
        let sided =
            parse_sided_declaration(r#"{ "format": "thaum-sided-declaration", "version": 1 }"#)
                .unwrap();
        assert!(sided
            .resolve(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg0))
            .is_none());
    }

    #[test]
    fn sprite_and_none_graphics_parse() {
        let sided = parse_sided_declaration(
            r#"{
                "format": "thaum-sided-declaration",
                "version": 1,
                "sides": {
                    "pos-z": { "sprite": "cell-sprites/proofs/chest.png" },
                    "neg-z": "none"
                }
            }"#,
        )
        .unwrap();
        match &sided
            .resolve(FacingRotation::new(CellFacing::PosZ, CellRoll::Deg0))
            .unwrap()
        {
            SideGraphic::Sprite(sprite) => {
                assert_eq!(
                    sprite.atlas_relative_path().to_str(),
                    Some("cell-sprites/proofs/chest.png")
                );
            }
            other => panic!("expected sprite, got {other:?}"),
        }
        match &sided
            .resolve(FacingRotation::new(CellFacing::NegZ, CellRoll::Deg0))
            .unwrap()
        {
            SideGraphic::None => {}
            other => panic!("expected none, got {other:?}"),
        }
    }

    #[test]
    fn retired_per_side_keys_reject_with_a_pointing_error() {
        for key in ["color", "weight", "shader-stack"] {
            let json = format!(
                r#"{{
                    "format": "thaum-sided-declaration",
                    "version": 1,
                    "sides": {{ "pos-z": {{ "glyph": "x", "{key}": 1 }} }}
                }}"#
            );
            let error = parse_sided_declaration(&json)
                .err()
                .unwrap_or_else(|| panic!("{key}: expected rejection"));
            assert!(
                error.0.contains("graphic-only"),
                "{key}: error should point at the cell-level move, got: {}",
                error.0
            );
        }
    }

    #[test]
    fn canonical_sample_file_loads_from_the_asset_root() {
        let root = std::env::var("THAUM_RENDERER_ASSET_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestration/renderer-assets")
            });
        let sided = load_sided_declaration(&root.join("sided/stick.json"))
            .expect("sample stick declaration loads");
        assert_eq!(
            resolved_glyph(&sided, CellFacing::PosZ, CellRoll::Deg0),
            '┃'
        );
        assert_eq!(
            resolved_glyph(&sided, CellFacing::PosZ, CellRoll::Deg90),
            '━'
        );
        assert_eq!(
            resolved_glyph(&sided, CellFacing::PosY, CellRoll::Deg180),
            '▪'
        );
    }

    #[test]
    fn serialization_round_trips_through_the_parser() {
        let parsed = stick();
        let json = sided_declaration_json(&parsed);
        let reparsed = parse_sided_declaration(&json).expect("serialized declaration reparses");
        assert_eq!(parsed, reparsed);

        // Same-as aliases survive as aliases, not expanded graphics.
        let json_text = sided_declaration_json(&parsed);
        assert!(json_text.contains("\"same-as\": \"pos-z\""));
        assert!(json_text.contains("\"same-as\": \"pos-z/deg-90\""));
    }

    #[test]
    fn serialization_carries_base_graphics_and_none_sides() {
        let sided = parse_sided_declaration(
            r#"{
                "format": "thaum-sided-declaration",
                "version": 1,
                "base": { "sprite": "cell-sprites/proofs/grass.png" },
                "sides": { "pos-y": "none" }
            }"#,
        )
        .unwrap();
        let reparsed =
            parse_sided_declaration(&sided_declaration_json(&sided)).expect("round trips");
        assert_eq!(sided, reparsed);
    }

    #[test]
    fn broken_declarations_reject_with_precise_errors() {
        let cases: Vec<(&str, &str)> = vec![
            ("wrong format tag", r#"{ "format": "other", "version": 1 }"#),
            (
                "unsupported version",
                r#"{ "format": "thaum-sided-declaration", "version": 2 }"#,
            ),
            (
                "unknown top-level key",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "side": {} }"#,
            ),
            (
                "unknown facing key",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "front": { "glyph": "x" } } }"#,
            ),
            (
                "unknown orientation key",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "orientations": { "pos-z/left": {} } }"#,
            ),
            (
                "missing graphic",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "pos-z": {} } }"#,
            ),
            (
                "graphic with two forms",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "pos-z": { "glyph": "x", "sprite": "a.png" } } }"#,
            ),
            (
                "same-as to undeclared side",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "pos-z": { "same-as": "neg-y" } } }"#,
            ),
            (
                "same-as cycle",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "pos-z": { "same-as": "neg-z" }, "neg-z": { "same-as": "pos-z" } } }"#,
            ),
            (
                "self same-as",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "sides": { "pos-z": { "same-as": "pos-z" } } }"#,
            ),
            (
                "orientation same-as to undeclared entry",
                r#"{ "format": "thaum-sided-declaration", "version": 1, "orientations": { "pos-z/deg-90": { "same-as": "pos-x/deg-90" } } }"#,
            ),
        ];
        for (label, json) in cases {
            let error = parse_sided_declaration(json)
                .err()
                .unwrap_or_else(|| panic!("{label}: expected rejection"));
            assert!(!error.0.is_empty(), "{label}: error message empty");
        }
    }

    #[test]
    fn portable_keys_round_trip_through_the_vocabulary() {
        for facing in CellFacing::ALL {
            assert_eq!(
                facing_from_portable_name(facing_portable_name(facing)),
                Some(facing)
            );
        }
        for roll in CellRoll::ALL {
            assert_eq!(
                roll_from_portable_name(roll_portable_name(roll)),
                Some(roll)
            );
        }
        assert_eq!(
            orientation_portable_key(FacingRotation::new(CellFacing::NegX, CellRoll::Deg270)),
            "neg-x/deg-270"
        );
    }
}
