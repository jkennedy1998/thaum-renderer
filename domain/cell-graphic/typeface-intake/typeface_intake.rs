//! Portable typeface declaration intake: parse one declaration file into a
//! runtime [`TypefaceDeclaration`]. This boundary owns the manifest format
//! (spacing flag, ordered native weight-file list) and its validation; it
//! does not load font bytes, rasterize glyphs, or resolve weight paths
//! against an asset root — that is `cell-graphic/glyph/`'s job, the same
//! split `sided-intake/` keeps from `facing-variants/` resolution.
//!
//! Weight-file paths are stored exactly as authored, never resolved here.
//! A relative path is understood by the consumer to live under its asset
//! root (a bundled typeface); an absolute path points outside it (a
//! user/system-supplied typeface this system loads but does not host).

use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Format tag every typeface declaration file carries.
pub const TYPEFACE_DECLARATION_FORMAT_TAG: &str = "thaum-typeface-declaration";
/// The declaration format version this parser reads.
pub const TYPEFACE_DECLARATION_VERSION: u32 = 1;

/// Whether a typeface's glyphs share one advance width (monospace) or carry
/// their own natural width (proportional). Declared by the typeface, not
/// derived from cell dimensions: cell width/height is a free-standing
/// per-file setting, never owned by or derived from typeface identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypefaceSpacing {
    Monospace,
    Proportional,
}

/// One parsed typeface manifest: spacing behavior and the ordered native
/// weight-file ladder, lightest first. Floor of one weight; no declared
/// ceiling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypefaceDeclaration {
    pub spacing: TypefaceSpacing,
    pub weight_paths: Vec<PathBuf>,
}

impl TypefaceDeclaration {
    /// The typeface's own native weight count, lightest-to-heaviest ladder
    /// length. Resolution-time weight clamping (`CellWeight::clamped_to_max`)
    /// snaps against `native_weight_count() - 1`.
    pub fn native_weight_count(&self) -> usize {
        self.weight_paths.len()
    }

    pub const fn is_monospace(&self) -> bool {
        matches!(self.spacing, TypefaceSpacing::Monospace)
    }
}

/// One parsed declaration failure, with an author-facing message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypefaceDeclarationError(pub String);

impl fmt::Display for TypefaceDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typeface declaration: {}", self.0)
    }
}

impl std::error::Error for TypefaceDeclarationError {}

/// Parse one declaration file's text into a runtime [`TypefaceDeclaration`].
pub fn parse_typeface_declaration(
    json: &str,
) -> Result<TypefaceDeclaration, TypefaceDeclarationError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TypefaceDeclarationError(format!("invalid json: {e}")))?;
    typeface_declaration_from_value(&value)
}

/// Load and parse one declaration file from disk.
pub fn load_typeface_declaration(
    path: &Path,
) -> Result<TypefaceDeclaration, TypefaceDeclarationError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| TypefaceDeclarationError(format!("cannot read {}: {e}", path.display())))?;
    parse_typeface_declaration(&text)
}

fn typeface_declaration_from_value(
    value: &Value,
) -> Result<TypefaceDeclaration, TypefaceDeclarationError> {
    let object = value
        .as_object()
        .ok_or_else(|| TypefaceDeclarationError("top level must be a json object".to_owned()))?;

    match object.get("format") {
        Some(Value::String(tag)) if tag == TYPEFACE_DECLARATION_FORMAT_TAG => {}
        Some(Value::String(tag)) => {
            return Err(TypefaceDeclarationError(format!(
                "unknown format tag \"{tag}\", expected \"{TYPEFACE_DECLARATION_FORMAT_TAG}\""
            )))
        }
        _ => {
            return Err(TypefaceDeclarationError(format!(
                "missing \"format\": \"{TYPEFACE_DECLARATION_FORMAT_TAG}\""
            )))
        }
    }

    match object.get("version") {
        Some(Value::Number(n)) if n.as_u64() == Some(TYPEFACE_DECLARATION_VERSION as u64) => {}
        _ => {
            return Err(TypefaceDeclarationError(format!(
                "missing or unsupported \"version\": expected {TYPEFACE_DECLARATION_VERSION}"
            )))
        }
    }

    for key in object.keys() {
        if !matches!(key.as_str(), "format" | "version" | "spacing" | "weights") {
            return Err(TypefaceDeclarationError(format!("unknown key \"{key}\"")));
        }
    }

    let spacing = match object.get("spacing") {
        Some(Value::String(tag)) if tag == "monospace" => TypefaceSpacing::Monospace,
        Some(Value::String(tag)) if tag == "proportional" => TypefaceSpacing::Proportional,
        Some(Value::String(tag)) => {
            return Err(TypefaceDeclarationError(format!(
                "unknown \"spacing\" value \"{tag}\", expected \"monospace\" or \"proportional\""
            )))
        }
        _ => {
            return Err(TypefaceDeclarationError(
                "missing \"spacing\": expected \"monospace\" or \"proportional\"".to_owned(),
            ))
        }
    };

    let weights = match object.get("weights") {
        Some(Value::Array(entries)) => entries,
        _ => {
            return Err(TypefaceDeclarationError(
                "missing \"weights\": expected a non-empty array of font file paths".to_owned(),
            ))
        }
    };

    if weights.is_empty() {
        return Err(TypefaceDeclarationError(
            "\"weights\" must declare at least one font file, lightest first".to_owned(),
        ));
    }

    let mut weight_paths = Vec::with_capacity(weights.len());
    for (index, entry) in weights.iter().enumerate() {
        match entry {
            Value::String(path) if !path.is_empty() => weight_paths.push(PathBuf::from(path)),
            Value::String(_) => {
                return Err(TypefaceDeclarationError(format!(
                    "\"weights\"[{index}] must not be empty"
                )))
            }
            _ => {
                return Err(TypefaceDeclarationError(format!(
                    "\"weights\"[{index}] must be a font file path string"
                )))
            }
        }
    }

    Ok(TypefaceDeclaration {
        spacing,
        weight_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const THAUM_MONO_JSON: &str = r#"
    {
      "format": "thaum-typeface-declaration",
      "version": 1,
      "spacing": "monospace",
      "weights": [
        "glyph-fonts/thaum-mono/ThaumMono-W80.ttf",
        "glyph-fonts/thaum-mono/ThaumMono-W160.ttf",
        "glyph-fonts/thaum-mono/ThaumMono-W320.ttf",
        "glyph-fonts/thaum-mono/ThaumMono-W640.ttf"
      ]
    }"#;

    #[test]
    fn declaration_parses_spacing_and_ordered_weight_ladder() {
        let declaration = parse_typeface_declaration(THAUM_MONO_JSON).unwrap();
        assert!(declaration.is_monospace());
        assert_eq!(declaration.native_weight_count(), 4);
        assert_eq!(
            declaration.weight_paths[0],
            PathBuf::from("glyph-fonts/thaum-mono/ThaumMono-W80.ttf")
        );
        assert_eq!(
            declaration.weight_paths[3],
            PathBuf::from("glyph-fonts/thaum-mono/ThaumMono-W640.ttf")
        );
    }

    #[test]
    fn proportional_spacing_parses() {
        let declaration = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "proportional",
                "weights": ["glyph-fonts/example/Example-Regular.ttf"]
            }"#,
        )
        .unwrap();
        assert!(!declaration.is_monospace());
        assert_eq!(declaration.native_weight_count(), 1);
    }

    #[test]
    fn a_single_weight_floor_is_valid() {
        let declaration = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": ["glyph-fonts/example/Example-Regular.ttf"]
            }"#,
        )
        .unwrap();
        assert_eq!(declaration.native_weight_count(), 1);
    }

    #[test]
    fn more_than_four_weights_is_not_rejected() {
        let declaration = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": ["a.ttf", "b.ttf", "c.ttf", "d.ttf", "e.ttf", "f.ttf"]
            }"#,
        )
        .unwrap();
        assert_eq!(declaration.native_weight_count(), 6);
    }

    #[test]
    fn external_absolute_weight_paths_pass_through_unresolved() {
        let declaration = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": ["/home/user/fonts/JetBrainsMono-Regular.ttf"]
            }"#,
        )
        .unwrap();
        assert_eq!(
            declaration.weight_paths[0],
            PathBuf::from("/home/user/fonts/JetBrainsMono-Regular.ttf")
        );
    }

    #[test]
    fn missing_format_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{ "version": 1, "spacing": "monospace", "weights": ["a.ttf"] }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("missing \"format\""));
    }

    #[test]
    fn unknown_format_tag_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{ "format": "nope", "version": 1, "spacing": "monospace", "weights": ["a.ttf"] }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("unknown format tag"));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 2,
                "spacing": "monospace",
                "weights": ["a.ttf"]
            }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("unsupported \"version\""));
    }

    #[test]
    fn missing_spacing_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{ "format": "thaum-typeface-declaration", "version": 1, "weights": ["a.ttf"] }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("missing \"spacing\""));
    }

    #[test]
    fn invalid_spacing_value_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "squiggly",
                "weights": ["a.ttf"]
            }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("unknown \"spacing\" value"));
    }

    #[test]
    fn missing_weights_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{ "format": "thaum-typeface-declaration", "version": 1, "spacing": "monospace" }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("missing \"weights\""));
    }

    #[test]
    fn empty_weights_array_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": []
            }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("at least one font file"));
    }

    #[test]
    fn non_string_weight_entry_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": [1]
            }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("weights\"[0]"));
    }

    #[test]
    fn unknown_key_is_rejected() {
        let error = parse_typeface_declaration(
            r#"{
                "format": "thaum-typeface-declaration",
                "version": 1,
                "spacing": "monospace",
                "weights": ["a.ttf"],
                "typo": true
            }"#,
        )
        .unwrap_err();
        assert!(error.0.contains("unknown key \"typo\""));
    }

    #[test]
    fn canonical_sample_file_loads_from_the_asset_root() {
        let root = std::env::var("THAUM_RENDERER_ASSET_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestration/renderer-assets")
            });
        let declaration = load_typeface_declaration(&root.join("typefaces/thaum-mono.json"))
            .expect("sample thaum-mono declaration loads");
        assert!(declaration.is_monospace());
        assert_eq!(declaration.native_weight_count(), 4);
    }
}
