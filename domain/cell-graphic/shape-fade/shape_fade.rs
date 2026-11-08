//! Shape-fade encapsulation (see `contract.md`): shape-space interpolation
//! between cell graphics. Any loaded glyph or sprite can fade with any
//! other: the gradient-tour walk sweeps through real interpolative glyphs
//! whose distance to the target is non-increasing, landing exactly on the
//! target — monotone toward the goal by construction.

pub mod compare_tile;
pub mod fade;
pub mod font_tiles;
pub mod mask_space;
pub mod neighbor_graph;
pub mod similarity;
