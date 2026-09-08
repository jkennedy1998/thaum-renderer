//! Shape-fade encapsulation (see `contract.md`): shape-space interpolation
//! between cell graphics. Any loaded glyph or sprite can fade with any other,
//! routing through interpolative glyphs in multiple steps when a direct blend
//! would look bad.

pub mod fade;
pub mod font_tiles;
pub mod mask_space;
pub mod neighbor_graph;
pub mod similarity;
