//! Shape-fade demo (J 2026-09-07): builds the real shape-fade graph over the
//! real staged typeface (Thaum Mono + sprite batches) and prints fade walks
//! for a set of character pairs so the routing feel can be inspected. Run
//! from the domain crate: `cargo run -p thaum-renderer-domain --example shape_fade_demo`.

use thaum_renderer_domain::shape_fade::fade::ShapeFade;
use thaum_renderer_domain::shape_fade::neighbor_graph::FadeTileProvider;
use thaum_renderer_domain::GlyphFontSet;
use thaum_renderer_domain::{CellWeight, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

struct FontTiles {
    font_set: GlyphFontSet,
    charset: Vec<char>,
}

impl FadeTileProvider for FontTiles {
    fn tiles(&self) -> Vec<(char, thaum_renderer_domain::GlyphTileRaster)> {
        self.charset
            .iter()
            .map(|&glyph| {
                (
                    glyph,
                    self.font_set.rasterize_glyph_tile(glyph, CellWeight::One),
                )
            })
            .collect()
    }
}

fn shade(alpha: u8) -> char {
    match alpha {
        0 => ' ',
        1..=63 => '.',
        64..=127 => '+',
        128..=207 => '*',
        _ => '#',
    }
}

fn render_tile(tile: &thaum_renderer_domain::GlyphTileRaster) -> Vec<String> {
    (0..GLYPH_TILE_HEIGHT)
        .map(|y| {
            (0..GLYPH_TILE_WIDTH)
                .map(|x| shade(tile.alpha[y * GLYPH_TILE_WIDTH + x]))
                .collect()
        })
        .collect()
}

fn print_fade(fade: &ShapeFade, from: char, to: char, steps: usize) {
    println!("=== {from} -> {to}");

    let mut frames: Vec<(char, Vec<String>)> = Vec::new();
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let resolved = fade.resolve_shape_fade(from, to, t).unwrap_or('?');
        let tile = {
            let graph = fade.graph_at_weight(CellWeight::One);
            match graph.index_of(resolved) {
                Some(index) => graph.tile(index).clone(),
                None => continue,
            }
        };
        frames.push((resolved, render_tile(&tile)));
    }

    // Header: the resolved char of each frame, centered over its 12-wide column.
    let walk: String = frames.iter().map(|(glyph, _)| glyph).collect();
    println!("walk: {walk}");
    let header: String = frames
        .iter()
        .map(|(glyph, _)| {
            let mut cell = vec![' '; GLYPH_TILE_WIDTH];
            let offset = GLYPH_TILE_WIDTH / 2;
            cell[offset] = *glyph;
            cell.into_iter().collect::<String>() + " "
        })
        .collect();
    println!("{header}");
    for row in 0..GLYPH_TILE_HEIGHT {
        let line: String = frames
            .iter()
            .map(|(_, rendered)| rendered[row].clone() + " ")
            .collect();
        println!("{line}");
    }
    println!();
}

const PROGRESS_STEPS: usize = 16;

fn main() {
    let asset_root = std::env::var("THAUM_RENDERER_ASSET_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../orchestration/renderer-assets")
        });
    let font_set = match GlyphFontSet::load_from_asset_root(&asset_root) {
        Ok(font_set) => font_set,
        Err(error) => {
            eprintln!(
                "failed to load the font set from {}: {error}",
                asset_root.display()
            );
            std::process::exit(1);
        }
    };

    let mut charset: Vec<char> = ('!'..='~').collect();
    charset.insert(0, ' ');
    let provider = FontTiles {
        font_set,
        charset: charset.clone(),
    };
    let mut fade = ShapeFade::build(&provider);
    println!(
        "graph: {} graphics, canonical weight One\n",
        fade.graph_at_weight(CellWeight::One).chars.len()
    );

    // The ten richest walks: each visits interpolative glyphs between the
    // endpoints (found by the FADE_SCAN2 sweep over the full charset).
    let pairs = [
        ('O', 'X'),
        ('7', 'J'),
        ('H', 'S'),
        ('D', 'Y'),
        ('5', 'F'),
        ('3', 'f'),
        ('%', '1'),
        ('.', ':'),
        ('R', 'u'),
        ('l', '%'),
    ];
    if std::env::var("FADE_SCAN2").is_ok() {
        // Scan: pairs whose resolved walk visits >= 3 distinct graphics.
        let chars = fade.graph_at_weight(CellWeight::One).chars.clone();
        let mut rich: Vec<String> = Vec::new();
        for &from in &chars {
            for &to in &chars {
                let mut seen: Vec<char> = Vec::new();
                for step in 0..=PROGRESS_STEPS {
                    let t = step as f32 / PROGRESS_STEPS as f32;
                    if let Some(resolved) = fade.resolve_shape_fade(from, to, t) {
                        if !seen.contains(&resolved) {
                            seen.push(resolved);
                        }
                    }
                }
                if seen.len() > 2 {
                    rich.push(format!(
                        "{from} -> {to}  walk: {}",
                        seen.iter().collect::<String>()
                    ));
                }
            }
        }
        println!("walks visiting 3+ distinct graphics: {}", rich.len());
        for description in rich.iter().take(60) {
            println!("  {description}");
        }
        return;
    }

    for (from, to) in pairs {
        print_fade(&mut fade, from, to, 6);
    }
}
