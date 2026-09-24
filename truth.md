# thaum-renderer design truths

### 2026-09-19-mole-dirt-bands-and-darkest-contrast-remap
- j-quote: "mole:\nf0ac90 lightest material\nc4702b mat light medium\n894835 darkmedium material\n3d2329 darkest material\n\ndirt: (same as mole for now, keep it separate as a material so it stays clean if we change colors. )\nf0ac90 lightest material\nc4702b mat light medium\n894835 darkmedium material\n3d2329 darkest material"
- interpretation: `Mole` and `Dirt` are distinct material ids with independently stored but currently identical four-band palettes.

### 2026-09-19-darkest-light-remap-retains-sprite-contrast
- j-quote: "brand black ( lightest medium, and light medium material, brand black on sprite)\ndarkest material (dark medium material on sprite)\ndark medium material (darkest material on sprite)\nlight medium material (brand white on sprite)"
- interpretation: only `CELL_SHADER_LIGHT_MINUS_3` remaps sprite indexed values: black/medium-light/lightest -> brand black; medium-dark -> darkest; darkest -> medium-dark; brand white -> medium-light. The other light states retain ordinary ramp shifts.

### 2026-09-19-mole-indexed-material-palettes
- j-quote: "120a19 darkest brand color\nfeffe5 lifghtest brand color (for all materials)\n\nwood: \n3d2329 darkest material \n81172a dark medium  material\na8561a light medium material\nea9827 light material\n\nstone\n3d2329 darkest material \n404863 dark medium  material\n787d8b light medium material\nc5b5a8 light material"
- interpretation: the complete indexed ramp uses the supplied brand endpoints around the supplied four-band wood and stone palettes. Renderer resolution exposes them as material ids; indexed light shifts continue to move through the same six-value ramp.

### 2026-09-11T18-30-59Z-painter-asset-bridge
- j-quote: "this asset bridge should probably not be the an integral part of the painter because the painter's lied to other people and they're going to roduce files so this should really just be like looking at files finding out how to turn them into assets."
- j-quote: "So we should keep the asset bridge pretty loose"
- j-quote: "I could see using this if I was building a program and wanted to dictate the animation of something and then save that and then bake it later. Then I can reuse it everywhere"
- j-quote: "maybe this is something that belongs in the renderer because a user downstream would end up using the ainter and the renderer hand to hand most likely"
- interpretation: A renderer-adjacent reusable bridge must read existing painter files as external source inputs and compile loose, reusable renderer-oriented assets; it must not become part of painter persistence or impose game-specific semantics.

### 2026-09-11T20-49-23Z-simple-bridge-transfer
- j-quote: "Just a file transfer system so we can have compressed down graphics that are easy to work with. They need a facing direction, a name, and to track the cell contents. They might have animation content mapped to breaths. It should be simple as can be."
- interpretation: The bridge should emit a minimal compressed transfer asset carrying a name, facing, cell contents, and optional breath-mapped animation.

### 2026-09-11T20-49-23Z-flat-rendered-export
- j-quote: "Lets do one export called \"flat\" and it can export the whole 3d cell with animation but no layers intact. Just what the renderer would render. Needs to include interpolations!!!"
- interpretation: The initial renderer-facing export is `flat`: the final renderer-resolved animated 3D-cell result, including interpolations and excluding source-layer structure.

### 2026-09-11T20-49-23Z-shared-cell-language
- j-quote: "painter will have sprites and glyphs, and materials and flat color. Those exist in the game as well. They should share a language."
- interpretation: Renderer and painter should use one consistent language for glyph, sprite, material, and flat-color cell values.

### 2026-09-11T20-49-23Z-named-layer-semantics-and-fixed-origin
- j-quote: "we wouldn't really be exporting collision we might be exporting layers that use their names to dictate what those layers mean"
- j-quote: "layers should all have like a an origin location like a point where the asset is like fixated from we're going to use that to rotate later."
- interpretation: Downstream game semantics can derive from exported layer names rather than collision data, and exported authoring layers need an invariant origin/pivot for later rotation.

### 2026-09-11T21-21-17Z-transferable-assets-first
- j-quote: "whats the first step do you think towards getting the game extracted? it definitly starts in the assets and getting those transferable"
- interpretation: The first extraction milestone is transferable graphical assets between painter and game.

### 2026-09-11T21-56-56Z-painted-ordered-cell-shaders
- j-quote: "while we are changing the file systems, shaders. can they also be stored yet? ideally i can paint shaders on. shaders will be addiitive and have ordering. it will be per cell as well"
- interpretation: Transferable painter assets carry an ordered additive shader stack on every cell; the renderer resolves that stack in its declared order.

### 2026-09-11T22-51-01Z-paintable-game-shaders-only
- j-quote: "yes lets exclude those. we should allow for specific shaders to be painted on. for instance, we will havce a shader in the game for things that are on fire in different increments. the different levels of fire shader will be able to painted on once they are made."
- interpretation: The game shader registry marks which shaders are paintable; editor-only shaders are excluded. Each fire-intensity shader is a distinct paintable registered shader once available.

### 2026-09-12T11-43-15Z-general-exporter-presets-and-filenames
- j-quote: "I think it should save the preset that the asset exporter uses. We talked about asset exporter as a third thing. Not renderer or painter."
- j-quote: "I'm thinking we need an exporter that is general. Not a thaumworld export, but more like a single-layer export so we know what kind of file it is. They won't have IDs quite yet, just filenames. IDs will happen in the game layer where we store these exports."
- interpretation: A separate general exporter, outside renderer and painter, consumes a generic selected preset and emits filename-based assets; downstream game storage assigns its own IDs.

### 2026-09-12T11-47-57Z-document-flat-export-section
- j-quote: "No look back in this chat. You are missing something every layer does not need a single export, It was for a flat export section."
- interpretation: The first general exporter profile is the document-level `flat` section: it emits the complete renderer-resolved document rather than automatically exporting each layer.

### 2026-09-12T12-11-49Z-flat-preserves-interpolation
- j-quote: "flat preserves interpolation."
- interpretation: The `flat` export representation retains document interpolation semantics rather than baking only discrete sampled frames.

### 2026-09-12T07-48-05Z-frame-cap-and-hue-drag
- j-quote: "PC is getting hot probably bc we don't clamp frame rate so it's just fucking going for rendering it all out. Clamp to 60."
- j-quote: "Color block needs to be able to drag not just scroll for the hue setter."
- interpretation: The reusable renderer window surface must cap presentation at 60 Hz, and its generic color block must support direct hue-slider dragging as well as wheel movement.

### 2026-09-12T13-25-00Z-facing-as-quantized-rotation
- j-quote: "facing on layers would allow us to do some crazy things, like track rotations. that would lead into the rotation property row nicely. i think rotations could be smooth if we did some raster intertpolation (we already have that) instead of aliasing."
- interpretation: The renderer's six CellGroupFacing cardinals are the quantized states of one layer rotation concept: a keyframed rotation track (painter side, same hold/interpolate/ease shape as move) resolves discrete steps to CellGroupFacing via with_facing, and interpolated frames blend cell content between the two bracketing facings through the raster interpolation seam rather than aliasing the grid. Viewer-relative shading stays a separate shader-stack concern: resolve_shaded_* eventually takes the view angle between camera orientation and group facing so paintable shaders can shade per viewing angle (the old game's per-view_direction art variants, generalized).

### 2026-09-12T13-35-00Z-united-facing-seam
- j-quote: "some cells may retain facing information with special cells that shade differently on different facing setups. like an asymmetrical object... it makes sense for these to have set cells per facing directions. like chests in the old game. layers probably need a facing in general as well so that they can get rotated."
- interpretation: The renderer owns the united facing seam: CellGroupFacing (six cardinals) is the single vocabulary; per-layer facing arrives on the CellGroup via with_facing (rotation carrier), and per-cell facing arrives as facing-variant assets — one referenced asset holding a set of cells per facing direction (old-game chests, generalized) — never as per-cell facing fields. An effective-facing resolution seam combines the group's facing with the camera's view orientation into one value consumed by both facing-variant resolution and future view-angle shader inputs in resolve_shaded_*. Old-game ViewDirection maps onto the cardinals through one owned mapping function.

### 2026-09-12T13-50-00Z-facing-piece-alignment
- j-quote: "there 100% needs to be both facing of camera, and facing of objects in the game... i need per cell facing but who should opt in? ... i also have per camera angle facing. this is necessary and will be sick."
- interpretation: The renderer owns the facing alignment: one CellGroupFacing vocabulary (six cardinals); camera facing from the existing camera/view-orientation; object facing arriving on CellGroups via with_facing; per-cell facing carried entirely by referenced sprite assets whose definitions optionally declare per-facing cell sets (same_as fallbacks) — cells never carry facing fields and never opt in. The single combination seam is resolve_facing_variant(group_facing, camera_facing, asset keying: object|viewer) -> cell set: object keying serves asymmetrical objects that turn (chests), viewer keying serves per-camera-angle art (two-sided signs, the "sick" piece). View-angle shader shading later reads the same resolved context. Build order: asset variant format + seam, layer facing, painter authoring/preview, rotation track (snap first), shader view-angle input.

### 2026-09-12T14-05-00Z-universal-cell-facing
- j-quote: "all levels of facing matter. camera effects a cells relative facing additively so things actually look 3d in each relative view"
- interpretation: Facing composition is the renderer's newest shared algebra: CellGroupFacing gains a compose operation over the cube rotation group (24 elements, integer tables, exhaustively tested). Relative facing for any cell = camera-inverse composed with the group's facing composed with the cell's own facing (cells carry a universal facing field, default PosZ, skip-if-default in files). Variant sprite assets are keyed by this composed relative facing — no object-vs-viewer keying declaration exists; both behaviors emerge from composition. The resolution seam consumes composed relative facing plus the asset's variant table and nothing else.

### 2026-09-12T14-20-00Z-cell-facing-encapsulation
- j-quote: "lets make sure were building these in seams that wont hurt uis later. this sounds like renderer level content tbh bc the renderer is 3d."
- interpretation: Facing math is renderer-owned and lives in a new domain/cell-facing encapsulation, sibling of the cell-weight/cell-warble family. It owns the canonical six-cardinal CellFacing type (re-exported by cell-group as CellGroupFacing), the roll-free rotation algebra (rotate/unrotate directions, pure point remap), the GlobalDirection mapping through coordinate-space/global-directions, and the single relative-facing resolution camera-inverse composed with group composed with cell. Cell slots, cell-group fields, camera view-orientation, and future cell-graphic facing-variants all consume this one algebra so no UI seam is ever re-derived. Implemented and exhaustively tested (identity, rotate/unrotate inverses over all 36 pairs, pinned axis tables, global-direction roundtrip, camera/group composition cases).

### 2026-09-12T14-30-00Z-cell-facing-slot-and-variants
- interpretation: Cell gained the universal facing slot (default PosZ, composed at render time, facing-invariant by default) alongside weight/texture/warble, and cell-graphic gained the facing-variants child owning per-facing sprite tables with same-as fallbacks keyed by composed relative facing. Opt-in lives entirely in the asset; cells keep plain sprite references. Same-as cycles are broken assets and resolve to the base sprite. Workspace 405 tests green.

### 2026-09-12T14-40-00Z-camera-facing-carrier
- interpretation: view-orientation now owns the camera-facing carrier: camera_facing_for_swing maps each camera swing to the world side the camera views FROM (opposite of its depth direction), and view_relative_facing composes camera with group and cell facing. The pinned convention: relative facing PosZ means the viewer sees the cell's front, so authored front art resolves naturally at the default and swung views. Camera roll is display-space and does not participate in facing resolution — a rolled screen still looks at the same side of an object. This completes the renderer-side chain: camera facing, group facing, and cell facing compose through cell-facing into one variant-lookup key.

### 2026-09-12T14-35-00Z-roll-in-the-facing-group
- j-quote: "this would give us some CRAZY abilities if we had roll and swing a part of the cell based and layer based systems. then we give the users lots of abilities + we give myself the ability to make art smaller than a cell if we compact in data in rolls and such. lots of 3d possibilities there. ie characters that can be multiple parts but also map correctly when viewed at all angles. heavy sprites > 3d system which people love now days"
- interpretation: Roll joins the facing group — the system extends from 6 direction actions to the full 24-element cube rotation group (FacingRotation = facing x roll). The facing-only vocabulary remains the read-mostly surface (roll defaults to 0); variant tables key by full FacingRotation so roll-specific art is per-asset opt-in via the same SameAs fallbacks. This unlocks authored cell roll, roll-aware camera resolution, multi-part characters that map at all angles, and art compaction through rotation instead of duplication — the heavy-sprites-behaving-as-3d direction.

### 2026-09-12T20-26-16Z-facing-variants-manual-authoring
- j-quote: "it will 99% come out of sprites and character models having to dictate body sizes"
- j-quote: "it will probagblty be provided and built manually in the paintyer to show what facing sides should look like if i have problems"
- j-quote: "a character model using a face sprite at a few different angles to get it looking really 3d"
- j-quote: "idk if ill do much of this just yet. its mostly for getting assets workign with this"
- interpretation: Facing-variant art is manually authored per orientation in the painter, primarily for sprites/characters viewed from a few angles; the FacingKey-vs-explicit-24-key lookup decision stays open and low-priority until asset work demands it.

### 2026-09-12T20-26-16Z-roll-serves-placed-asymmetric-objects
- j-quote: "there going to be pieces that are going to be asymmetrical like a chest for instance and when we roll them we might want to adjust the graphic to something that's pre planned for that role location"
- j-quote: "for things that are semi Symmetrical as us declaring sides as a shared graphic is like an upper side"
- j-quote: "with more complex objects like asymmetrical creatures That's what this is for"
- j-quote: "It's probably going to be something like sprites but one level deeper that's kind of how I thought of it in my head but since we just did a bunch of this work with getting Roll integrated into self we might not have to go that route anymore"
- interpretation: Roll-driven graphic swaps serve placed asymmetrical objects (chests, creatures) with pre-planned per-roll graphics; semi-symmetrical things share side graphics via declared sides; the cell roll integration may remove the need for a deeper sub-sprite mechanism.

### 2026-09-12T20-26-16Z-painter-displays-roll-through-camera
- j-quote: "The painter shows rolled already via camera controls"
- j-quote: "the painter should probably be able to show some sort of Sprite or something that is set up for multiple camera angles / rolls. it's going to be consumed less than tailored on the app for painting"
- interpretation: The painter eventually displays facing/roll variant resolution through its camera as a layer-level thing; authoring is asset-level and the painter consumes the renderer's resolution rather than tailoring it for painting.

### 2026-09-12T20-26-16Z-flat-export-preserves-roll-metadata
- j-quote: "flat I just meant compressing all the layers down I think that something like roll would end up being preserved but I would approach this in the most simple way you can"
- j-quote: "if I made a tile that's like stone bricks on all sides except for one of the 5 sides maybe it is a crack on it"
- j-quote: "have a tile in game that can get rendered with different graphics per size and have a facing so that we can change the angle of that object"
- interpretation: Flat export flattens layers but preserves facing/roll as simple carried metadata so the game re-renders per-side graphics and re-orients objects; target uses are chests, characters, and per-side tiles like a cracked stone brick.

### 2026-09-12T20-36-00Z-per-part-facing-on-multi-cell-objects
- j-quote: "if a character isn't just a cell big what if they're let's say 2 cells a head and a torso. The head has a facing the torso might be a different facing"
- j-quote: "what if we had a big snake that was made out of boxes and we wanted to display its head and its tail and midsections each of those might have a real facing graphic that could change and tell the user about how the Snape is positioned"
- j-quote: "Those graphics might just be glyphs Like we might just assign each facing side to a glyph or a Sprite that's essentially how it's going to work it's going to be like AA router for like each facing side And kind of like the exceptions of like on this side I'm actually going to be this graphic"
- j-quote: "I think that should extend to not just graphic but weight and material as well"
- interpretation: Facing variants apply per part of multi-cell objects (each snake segment / head / torso resolves its own facing); the variant router assigns a glyph or sprite per facing side with per-side exceptions, and the resolved value should eventually cover weight and material channels, not just graphic.

### 2026-09-12T20-48-00Z-variant-key-is-full-orientation-with-fallback-ladder
- j-quote: "option A \"North side, rolled 90°\" -> table needs up to 24 entries this is right"
- j-quote: "it doesnt always needs this. somsetimes a sprite will just have 1 per all sides. sometimes no roll. i need to support all ideally cleanly throuigh 1 way"
- interpretation: The variant lookup key is the full 24-element orientation (side + roll); one mechanism supports every specificity level through a fallback ladder — exact orientation entry, then facing-only entry (roll-agnostic), then base sprite — so an asset declares only as much variant art as it actually has.

### 2026-09-12T17-18-00Z-sided-thing-is-a-third-graphic-kind
- j-quote: "i think a 3rd thhing like glyph and cell are. its different tho. i need to declare a rendered graphic, weight and material/ color for each side independently. then a 6 sided 3d cell can rotate using the cell facing and we start to make sense of this all."
- interpretation: The side-declaring thing is a first-class third graphic kind, sibling to glyph and sprite (J explicitly chose this over the model-leaner "sprite asset with a variant table"). Each side entry carries its own full appearance — graphic, weight, AND material/color — not just a sprite reference. A placed cell's facing selects the side.

### 2026-09-12T17-18-00Z-sided-key-space-is-sides-plus-optional-rolls
- j-quote: "WAIT but it was like a 26 sided system bc of roll. so this 3rd thing really needs to be able to declare 6 sides, and possibly custom rolls..."
- interpretation: The third kind's key space is the 6 sides with roll extensions possible per side — this is exactly the 24-element orientation group (6 facings x 4 rolls; 24, not 26) already implemented as the ladder: declare facing-tier entries for the sides, add exact-orientation (side+roll) entries only where custom roll art exists.

### 2026-09-12T17-18-00Z-sided-declared-alongside-sprites-programs-declare-own
- j-quote: "probably in the same place as sprites are declared. iots a renderer thing, programs can end up declaring their opwn sprites or 3dcell objects."
- interpretation: The third kind is declared at the renderer asset layer, in the same declaration place as sprites; consuming programs/games can declare their own sprite and sided-3d-cell assets rather than the painter owning the vocabulary.

### 2026-09-12T17-25-00Z-sided-is-the-name
- j-quote: "1 sided makes a lot of sense."
- interpretation: The third graphic kind is named `Sided` (kind) / sided object (prose), sibling to glyph and sprite in CellGraphic.

### 2026-09-12T17-25-00Z-side-look-is-full-appearance
- j-quote: "2 ordered shader stack per cell, col/mat, sprite/glyph, weight per side"
- interpretation: Each side entry declares a complete appearance: ordered shader stack, color/material, graphic (sprite OR glyph), and weight — independently per side. The ladder's base tier is the cell's own authored appearance when a side is undeclared.

### 2026-09-14-overlap-becomes-layering-at-same-xyz
- j-quote: "Right now when you author a cell group and it contains coordinates that are of like another cell group's contents what happens is the cell group that was rendered last overrides the previous cell groups like it deletes or clears out that cell in a way not in a way that changes the data but right now it doesn't render in layers. I think that as I'm developing these things I would enjoy if I was able to instead of replace graphics with those rendering layers is if they were layered on top of each other but only if they shared the same XY and Z coordinate not just layers They're on top of each other. This is specifically in the overlap rule when two cells of different groups occupied the same XYZ space for the global cell MAP"
- interpretation: overlap-policy changes from last-wins replacement to LAYERING: cells from different groups at the same exact world XYZ stack (later pass on top of earlier), instead of erasing the earlier cell. Layering is same-XYZ only — it is not general z-layering of unrelated depth. Implementation home is domain/composition/overlap-policy (compose_cells returns stacked cells per key); the engine bridge gains the right to emit overlays (fire, highlights, damage tints) as separate groups without pre-merging. OPEN QUESTIONS before implementation: (1) is pass order the bottom-to-top layer order? (2) does the lower layer show through upper-layer transparent pixels, or do layers blend colors? (3) which layer drives lighting/occlusion when stacked? (4) does each layer carry its own weight lane value?

### 2026-09-14-overlap-layering-semantics-ratified
- j-quote: "One I assume that later groups are on top because they are rendered last and that language makes the most sense to me"
- j-quote: "Yes they should show the cell behind them with true alpha stacking within that one cell that shares the XYZ coordinates. That would be the best implementation of this. Not extra quads just alpha stacking cleanly done"
- j-quote: "In a stack coordinate both should drive lighting and occlusion on the data side On the data side you should not overwrite each other like the rendering side does or it used to Ideally we could take into account any keep tile or character on any cell. Not all characters may collide not all cells will collide not all heaps will collide so we could have really a lot of different scenarios of different amounts of items on a cell at a given time. Characters collide by default tho. its more so a case for ghosts and things like that lol"
- j-quote: "lighting will end up effecting the material shading, changing the indexed colros that route to different RGB channels / preset colors. we built this partially in the old system with the RGB stuff, and i think it even works in the new renderer -- we just haavent really tested that as muich"
- j-quote: "i dont realyl have a way to author more than one material in the painter atm but i think they can render as 3 different like the old system used to"
- j-quote: "weight per cell for rendering as in type weight. not correlated with actual weight rn. fire making somethbing breath effects its weight as a shader ideally, that fire can add automatically to tiles, hjeaps or charcters that are one fire / have the fire tag"
- interpretation: overlap is now LAYERING (landed in compose_cells): pass order = bottom-to-top, true alpha stacking per shared XYZ, data side aggregates every layer for lighting/occlusion. Weight is TYPE weight (visual), never sim weight; fire-as-shader is the renderer's job — a fire shader adds weight/breath automatically to anything carrying the fire tag, the engine only names the shader (APPLY_SHADER stays a name pointer). Lighting routes through material shading (indexed colors → RGB channels / preset colors); painter currently authors one material per definition but old system rendered 3 — multi-material authoring is a painter-side gap, noted not solved.

### 2026-09-14-layering-landed-in-the-render-path
- interpretation: layering is now real end to end. Two dedupe points existed and both are gone: (1) domain/composition compose_cells stacks cells per world XYZ in pass order; (2) orchestration/boot stage_projected_boot_cells stages every layer in pass order — no projected-position overwrite, quads extend in pass order so later groups paint on top, true alpha show-through via blending. Cells whose shader hides them this frame produce no quads (quad-time visibility), so lower layers show through instead of blanking — the old flashing-overlay yield behavior emerges naturally from layering. Proofs: composition stacking tests (2), boot staging layer-order tests, scene-level two-quad overlap proof (red beneath, green on top).

### 2026-09-19-seven-state-indexed-lighting
- j-quote: "brighter and brigtest just keep following the same pattern."
- interpretation: lighting shifts the full program ramp — brand black, four material bands, brand white — by -3 through +3. The seven resulting states are darkest, dim, shadow, lit, bright, brighter, and brightest; endpoint saturation performs the intended color compression.

### 2026-09-19-brand-black-is-16110e
- j-quote: "16110e lets get alligned on this color. this should be brand black / darkest color in the set."
- j-quote: "3d2329 2a2a41 2d3a26 are one step up from those. i think these are all in the ascii painter. when tiles are totally black, the darkest brand color should be 16110e not 2a2a41"
- interpretation: the program's brand black (PROGRAM_BLACK, and the painter's off_black) is #16110e; the painter's deep_red/deep_green/deep_blue (#3d2329/#2d3a26/#2a2a41) are the material darkest bands one step above it. The old PROGRAM_BLACK (#120a19) and painter off_black (#120a1a) are superseded.

### 2026-09-19-sprite-quads-run-merge-when-untextured
- interpretation: sprite tiles with no texture/warble effect merge horizontal same-color texel runs into one wide quad (UVs span the run); the texture/warble path keeps the per-texel expanded footprint. Root cause: ground sprites (soil strip on every floor cell) emitted ~768 quads per cell and tanked llvmpipe to ~20fps (perf.jsonl: 14.8k -> 94.5k avg quads at 2026-09-19 11:29). The long-term shape is whole-cell atlas-sampled sprite quads like the glyph path; run-merging is the bounded step now.

### 2026-09-20T11-57-25Z-camera-transition-polish
- j-quote: "I'd like to polish out perspective and smoother transitions with roll and swing on the renderer side. We used to have this in the old system and it mostly worked well. I'd like to see what we can do to bring that into the renderer for swings and rolls."
- interpretation: renderer camera presentation should regain smooth roll and swing transitions while preserving its discrete six-view/quarter-roll semantic camera and its existing perspective system; this is renderer-owned work, not game-specific camera behavior.

### 2026-09-20T12-04-33Z-soft-freeform-camera
- j-quote: "I think number 1 sounds the best. Then the user can put the camera at different angles and it's less jarring."
- interpretation: use a soft-freeform renderer camera: continuous presentation angles over discrete 24-way semantic camera frames, so users can rest the view between frames without disturbing grid, art, or input truth.

### 2026-09-20T12-20-43Z-focus-ray-drives-renderer-facing
- j-quote: "Ideally this system detects the angle that the camera is looking at the point of focus and that angle is what drives where the renderer says that the user is facing. Keep in mind that this renderer has multiple users over multiplayer sessions. The 6 facing directions and 4 rolls are not going to change downstream, the rendering of cells relies on them"
- interpretation: every per-user camera derives its discrete renderer-facing `CameraSwing` and `CameraRoll` from its own continuous eye-to-focus pose, while retaining the existing 24-state output as the downstream projection and cell-variant contract; no renderer-global camera/facing state is permitted.
