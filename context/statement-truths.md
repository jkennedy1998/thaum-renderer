# /home/j/Repos/thaum-renderer/context/statement-truths

## purpose
Keep lightweight quoted source truth from J in exact words so renderer design can stay grounded against the questions that produced those answers.

## source
- extracted from `domains/workshops/operator/artifacts/message-logs/2026/2026-07-11.md`
- extracted from `domains/workshops/operator/artifacts/message-logs/2026/2026-07-12.md`
- focused on the renderer-shaping question/answer run from the last ~40 operator-room messages
- operator message logs preserve the room channel id but do not preserve per-message Discord ids
- channel-id: `1515722862156189767`

## note
- format here is question -> truth
- quotes keep J's wording rather than rewriting into summaries
- `discord-message-id` is unavailable in the operator log artifact

## renderer-scope-and-boundary

### question-1
> Should thaum-renderer own only render truth, while document authoring truth lives outside it?

### truth-1
> thaum renderer only owns what the rendering is doing + the shaders that go into the rendering. the art direction really of using a monospace ascii system with a linked sprite system. this particulaar project is tuned to thaum mono which is 12x16 and has 4 weights, and the sprite systems will have 4 weights too.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-2
> Should CellGroup be only a render-space layer/group, or also the canonical authored object container?

### truth-2
> a render space layer / grouping. intended for an entire module and relevant contents, one menu piece, one encapsulated display module, ect. the game / ascii painter will handle their own grouping of intra module groups. this is mostly just a rendering piece.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-3
> Do you want authored data and resolved render data as separate encapsulations?

### truth-3
> i dont think we should go in with that approach without thinking about it first. that is quite abstract. what are those doing in separate places? why? can it be simpler and cleaner?

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

## projection-camera-and-tools

### question-4
> Should projection be a first-class encapsulation separate from renderer core?

### truth-4
> projection as in like raycasting and projecting shapes from points, that sounds like a tool of the highest level owner of the matrix. may be out of scope of the thaum renderer, who deals in matrix stuff, but is not a good place to put matrix bound tools. maybe tools/matrix-math as an organizational encapsulation for something like tools/matrix-math/raycast or tools/matrix-math/projection to live.
>
> camera projection / the 3d views are owned by the camera, which is probably owned by the rendeer because you need  a camera to render.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-5
> Should camera/view state live outside renderer core, with renderer only consuming resolved view inputs?

### truth-5
> camera should live within the thaum-renderer maybe in tools/camera/camera-controls or domains/camera/roll or domains/camera/swing. matrix stuff happens there that is complex to the renderer. (design work goes in domain, repeatable boundaries that are low level repeat work go in tools for multiple users to utilize. those can expose the ability to roll or swing, but roll and swing can be defined elsewhere in domain )

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-6
> global camera placement and ownership details?

### truth-6
> global camera (what is being looked at via the thaum-renderer) should have a position on the global coordinates so it can be panned, will be able to move (snaps to the typegrid for a focused (relative)depth layer), roll and swing are camera things. visible depth range can be calculated as a camera tool thats a good ownership. maybe in camera/tools since its such a simple calculation from the camera point to the other world cooridnate as a distance. some (many) mopdules will own their own panning of some of their cell-group contents. local panning is owned by the general module features encapsulation which should outline a repeatable module usecase.
>
> modules are what will be used to link a cell group. i dont think the renderer has to know about a module.

- timestamp: `26 07 11 21 12 21`
- discord-message-id: `unavailable-in-operator-log`

### question-7
> Should camera be split as domain/camera and tools/camera/*, or stay as one domain first?

### truth-7
> i think sp[litting design work from general tool stuff is quite handy when needed. we can see in drafting itf that makes sense still once we see all the folders in place. an easy ammendment while in the contract/ folder plan phase.

- timestamp: `26 07 11 21 17 01`
- discord-message-id: `unavailable-in-operator-log`

## cell-color-and-shaders

### question-8
> Should cell-color own both flat color and shader refs, or should shader refs become their own sibling later?

### truth-8
> yes it owns both. domain/cell-color/shaders and domain/cell-color/flat-color would be good.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-9
> shader system direction and richness?

### truth-9
> yes thats a good start for the shader system. it needs access to much info, kind of like how blender texture coordinates can drive so many usefull things.

- timestamp: `26 07 11 21 12 21`
- discord-message-id: `unavailable-in-operator-log`

## animation-and-breath

### question-10
> Should cell-graphic own animation/frame selection, or should time/state choose graphic before render?

### truth-10
> cell-groups / cell stuff should probably not know about animation, as animaiton and moving things and rendering directions should come from elsewhere. cells are just pixels. animation is above them. cell-color is just one aspect of the pixel. animaiton is far above it.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-11
> Do you want animation as part of renderer v1 contracts now, or kept outside until render core is locked?

### truth-11
> as said above. animation should not be part of the renderer in a very complex way.
>
> we have a very very used system of "breath" that is used to give caidence to the whole system. the renderer will have to know about breath, and be able to be fed breath as an integer as shaders will use it. it shouldnt own anything like this as this renderer should just be simple data, it will be used in multiplayer applications.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-12
> Should breath enter through renderer as A) one global integer input or B) global + per-group offsets for shader use?

### truth-12
> A sounds fine.

- timestamp: `26 07 11 21 17 01`
- discord-message-id: `unavailable-in-operator-log`

## coordinate-space-and-cell-group

### question-13
> Should there be a domain/global-coordinate encapsulation, or is that owned directly by renderer?

### truth-13
> yes that sounds good.

- timestamp: `26 07 11 21 17 01`
- discord-message-id: `unavailable-in-operator-log`

### question-14
> Should the renderer own the coordinate system used for displaying the cell groups?

### truth-14
> yes i think it needs to be the owner of the coordinate system used for displaying the cell groups. cell groups have their own local coordinate system which is more used by local users of the cell groups / modules. global coordinates will get used in shaders though for screen space coordinates, for global world coordinates that help blend all cell groups together, and then for positioning the cell groups.

- timestamp: `26 07 11 21 12 21`
- discord-message-id: `unavailable-in-operator-log`

### question-15
> Should CellGroup transforms be local-to-parent only, with renderer composing globals, or should groups store global transform too?

### truth-15
> cell groups should use a global renderer coordinate system that can be traversed. some cell groups can be traversed but their own cell and world within the cell groups will just bve controlled and owned by the cell group that is being used to display them.
>
> the transform of cell groups should track position for xyz, size for xyz, and facing as a 6 coordinate global tracker. this will help us do easier matrix math that involves rotations.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

### question-16
> Should cell-group own its sparse storage contract directly, or should sparse indexing be a child like cell-group/storage?

### truth-16
> cell group can own its own sparse storage contract. that seems clean and like a good way to make sure everything works the same and is optimized under the hood.

- timestamp: `26 07 11 21 17 01`
- discord-message-id: `unavailable-in-operator-log`

### question-17
> sparse storage direction under real load?

### truth-17
> it should really be sparse / the best format for this when things get heavy. this is a big design question. we need to be able to render massive blocks of filled in cells, and also sparse / not very populated cell groups that might span a larger space.
>
> this was a bottleneck of the old system and is the main reason for a wanted rebuild.

- timestamp: `26 07 11 21 12 21`
- discord-message-id: `unavailable-in-operator-log`

### question-18
> facing / orientation constraints?

### truth-18
> just facing the 6 cardinals. that is part of the art direction because we use sprites. they always have to face, so we cut soft turns down to specific camera movements + a matrix rerendering so we can display things in 3d.

- timestamp: `26 07 11 21 12 21`
- discord-message-id: `unavailable-in-operator-log`

### question-19
> When two cell-groups overlap at exact xyz, should order be A) explicit render order list B) creation order fallback C) both?

### truth-19
> A, i dont like fallbacks when we can avoid them.

- timestamp: `26 07 11 21 17 01`
- discord-message-id: `unavailable-in-operator-log`

### question-20
> overlap behavior at exact shared xyz?

### truth-20
> overlap should be this. cell groups rendering on the same cells > the most recently rendered cell group can take presidence like a layer ordering system but with cell groups. only for cells that occupy the same exact xyz space though between two cell groups. this is 3d, so we can see all the given rendered cells ifg they dont occupy the same exacty xyz space.

- timestamp: `26 07 11 21 01 28`
- discord-message-id: `unavailable-in-operator-log`

## repo-shape-meta-truths

### truth-21
> you just crashed.  look over the discord. i deletedf two folders (composition and design) because they were unrelated tro the development shape i wanted.
>
> go over the last 30 or so discord messages that refine the shape of this development of the /home/j/Repos/thaum-renderer/ repo.
> its a full rebuild of the renderer oif the ascii system in the thaumworld auto styoryteller (bloated repoi)
>
> i want to continue scaffolding this out in fodlers and contract.md files

- timestamp: `26 07 11 21 21 25`
- discord-message-id: `unavailable-in-operator-log`

### truth-22
> runtime folders should not exist.
> orchestration is boot
> tool is reusable simple seams
> domain is design work and flow.
>
> i moved a few small things around.
>
> looking at the renderer in the older system, what else is missing?

- timestamp: `26 07 11 21 28 57`
- discord-message-id: `unavailable-in-operator-log`

## post-effects-and-cell-effects

### question-23
> Should `shader` be strictly `char + color + weight only`?

### truth-23
> 1 yes

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-24
> Should full-frame effects live in a separate seam like `post_fx` / `image_fx` / `finish_fx`?

### truth-24
> 2 lets do post_fx with just indexed-color as the only consuimer. maybe depth of field and positional displacement could be per cell. we have tot alk about this and figure it out if its possible per cell. (better fidelity.)

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-25
> Should indexed color clamping be always last in a post-fx stack?

### truth-25
> 3 always last in a post-fx stack. post-fx can be an encapsulation owned domain/

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-26
> Should blur live only in full-frame/post, never per-cell?

### truth-26
> 4 idk per cell could be sick

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-27
> Should warble exist in two layers: `cell warble` and `frame warble`?

### truth-27
> 5 idk only per cell could be sick

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-28
> Should cell warble be allowed to sample world data like `depth / water / heat / breath / tags`?

### truth-28
> 6 yes it needs/wants this because it would look amazing but this is per cell....

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-29
> Should breath continue as the primary animation clock for paused-state motion?

### truth-29
> 7 yes breath is the primary clock for the painter + thaumworld game. its a piped variable that we talked about earlier. those coulkd be used by shaders or post process i guess / cell process if that emerges.

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-30
> Should built-ins be split into two families: `shaders` and `effects`?

### truth-30
> 8 im thinking so yes, because shaders are color weight and graphic, but now blur and vector displace could be per cell too....

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-31
> Should UI styling prefer semantic presets over bespoke effects?

### truth-31
> 9 these should be not in the renderer. these sound like things other boundaries can hold. lets keep listed shaders light for now. maybe under 3 of them, for testing purposes and showing the format.

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

### question-32
> Should region-scoped effects be a first-class concept?

### truth-32
> 10 i think we pass things like this through the 3d matrix that the renderer gives exposure to. these are per other application, wont be a part of everything. we just leave pipelines for general use

- timestamp: `26 07 12 00 27 44`
- discord-message-id: `unavailable-in-operator-log`

## renderer-first-follow-up-pass

### question-33
> What else from the old system still smells renderer-first?

### truth-33
> 1: checker binary can be disregartded. 
> sin can be a shader that does a relative weight add and subtract.  basically just a scrolling via the new breath data channel (see below) .. -1,0,+1,0,-1... modulator on weight of cells through the new shader system on xyz coordinates globally.
>
> 2 world_z was bc xyz was not standard. we have world xyz now and local xyz
>
> focus_world_z is a messy name. its the depth of the reletive facing angle of the camera that is being focused on. should rework that name in the new system. dont think we added that one, good catch. this has to be a single source of truth, its related to the camera. the camera should own that focus target which has an xyz coordinate globally. the depth is the facing angle axis distance to that correct point coordinate.
>
> place_base_z nope dont include that. this is going to be owned by apps that use it. not in renderer.
>
> breath index (new change here) lets create a channel for this specifically. lets call it time, it can be an int. this value will be used in some of the standard shaders. like sin, it should be driven by this value. lets call that value in the channels "breath" and it should be an int.  if a breath link value is un set this value should fallback to something simple like increase by 1 every .25 seconds, modulus loop at a high value to avoid overflow. that fallback should only be used when apps do not provide a breath value on boot. that sounds like a worker for the breath management. maybe thaum-renderer/workers/breath-fallback-clock/ for this to live for the timing.
>
> light_mag is an int channel that will be in thaumworld game and thaum painter, not in renderer. 
>
> tile neighbors is conenction, we have that listed in the system already. 
>
> 3 we have this. its cell-graphics 
>
> 4 we have the dumb / fast version of this outlined in here already. 
>
> 5 we have this in composition
>
> 6 this is in app behavior. raster cell math is not renderer boundary. that is outside and not needed right now. this only renders and has to do with rendering, the cell-group inputs should already be formatted as they shoulda for display, into this renderer. 
>
> 7 this is included in cell-color already, and the override is cell-shaders
>
> 8 yep this is in atlas-intake
>
> 9 this is a real miss. created folders here. /home/j/Repos/thaum-renderer/domain/post-effects/
> self explanitory. these are the post process stack. bloomn and indexed color clamp can be the only two for now. the rest have been started as cell things like cell-warble and cell-blur. 
>
> 10 not owned by renderer actually. let programs do their own lighting and rout the light through a data channel, materials have access to those data channels and therefore can be custom lit via custom materials usiong the accessible data.

- timestamp: `26 07 12 15 38 26`
- discord-message-id: `unavailable-in-operator-log`

### question-34
> Follow-up status on `shader data intake`, `cell blur`, and `cell warble` before git?

### truth-34
> i removed shader data intake
>
> cell blur and cell warble are real and will be used. were figureing out if they work tho first.
>
> i think we can start a git for this repo of the thaum-renderer.

- timestamp: `26 07 12 15 44 32`
- discord-message-id: `unavailable-in-operator-log`

## data-delivery-and-built-in-shape

### question-35
> Should we name the middle seam `cell_fx`?

### truth-35
> 1 yes i think that makes the most sense.
> region data doesnt come through here, it goes through the xyz data delivery. that delivery method is not shader specific, its renderer specific -- and components of the renderer like cell-shader and cell-effects can consume it.

- timestamp: `26 07 12 00 52 45`
- discord-message-id: `unavailable-in-operator-log`

### question-36
> Should blur + warble be excluded from `post_fx` for now?

### truth-36
> 2 yeah lets build those in cell effects. each effect should be an encapsulation within there. same sort of pattern with shaders. letting programs define the cell effects and shaders should be fantastic. just providing a few default ones. this should be the same thing for materials, shaders, and cell effects, and post effects. those all can be set in form by the renderer, by allowed for consumers of the renderer to produce their own in an expected space (not within the renderer)

- timestamp: `26 07 12 00 52 45`
- discord-message-id: `unavailable-in-operator-log`

### question-37
> Should the initial built-in set be `checker`, `sine_weight`, and `indexed_color_clamp`?

### truth-37
> 3 the inital shaders can be "weight-sinxyz". the initial post effect can be indexed-color-clamp, the initial cell effect can be blur
>
> lets get this scaffold made! then we can refine it

- timestamp: `26 07 12 00 52 45`
- discord-message-id: `unavailable-in-operator-log`

## sprite-color-material-and-glyph-truths

### question-38
> Should the renderer treat these 24 colors as a strict canonical sprite input palette?

### truth-38
> 10 (your second message) yes I'll provide them in the form of pngs so compression shouldn't be an issue.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-39
> For flat colors, should mixed slots (`A+B`, `B+C`, `C+A`) also resolve as 50% blends of the assigned flat outputs?

### truth-39
> 11 yes.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-40
> For materials, should mixed slots blend the final resolved band colors after each material computes its own 4-band gradient?

### truth-40
> 12 yes

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-41
> Should every material always expose exactly 4 bands: `darkest / medium-dark / medium-light / lightest`?

### truth-41
> 13 yes always 4 bands, no interpolation needs to exist.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-42
> When a slot is assigned a flat color, should all 4 value-band source pixels collapse to exactly one output color with no light influence?

### truth-42
> 14 yes. (You said light. The renderer does not know about light because that is app specific. Not all apps will have a light system. That will be delivered in the form of an xyz input for thaumworld game and thaum painter)

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-43
> Should shaders be allowed to remap slot assignment at runtime: for example `A=metal`, `B=leaf`, `C=flat white`?

### truth-43
> 15 yes! Maybe shaders should also handle the per cell blur and per cell warble as well so they can derrive those effects (now listed as effects) from tags later.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-44
> Where should program-defined materials live?

### truth-44
> 16 in domain/cell-materials + 1 expected slot for the consumer program (did this planning a little already)

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-45
> Should `gray-scale` be the first built-in test material in the renderer contracts?

### truth-45
> 17 yes

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-46
> Should sprite import/validation fail hard if a pixel uses a non-canonical palette color?

### truth-46
> 18 no nearest match no warn.
> This newest color need can be in tools/color. Intake a color, and a collection of colors, output the nearest color in collection.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-47
> Do you want the sprite format to support `unused slot` semantics, or should A/B/C always be considered present even if absent in pixels?

### truth-47
> 19 they should always be present even if not in pixels.  One note I'm thinking of . Glyphs are not like sprites but we don't want to have to have so many different color systems. Glyphs need materials too. Their graphic system is currently in binary (black and white). We probably need an assumption for rendering sprites in black and white  (for alpha sprites) so this doesn't break / multiply the size of the system.

- timestamp: `26 07 12 12 10 33`
- discord-message-id: `unavailable-in-operator-log`

### question-48
> For glyphs, should black/white source graphics resolve as `black = transparent` and `white = slot A band-picked`?

### truth-48
> 20 yeah this sounds good. Let's make them medium light if they are assigned materials.
> This will change later.

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-49
> Should glyph binary mode instead map `black = slot A` and `white = slot B`?

### truth-49
> 21 no

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-50
> Should glyphs have their own declared color-space under `cell-graphic/glyph/` while sharing the same material resolution system as sprites?

### truth-50
> 22 yes but it should resolve to the same resolution shape as the sprite ones.

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-51
> For alpha-style sprites, should we support a canonical grayscale/mono source mode that the renderer expands into slot A + 4 bands?

### truth-51
> 23 yes but just pick one of 4 bands. Medium light.

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-52
> For the 8 generic data intakes, is this right: `2 bool`, `2 float`, and `4 existing generic lanes`?

### truth-52
> 24 yes

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-53
> Should nearest-color matching happen at import/load time, at runtime decode, or both allowed?

### truth-53
> 25 c. It will be used in post process to index color match in the effect later, then also be used for internal sprite stuff if needed. (It needs to be fast in both cases)

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-54
> For program materials in `cell-materials`, should the consumer slot be `cell-materials/program/`?

### truth-54
> 26 no the programs materials should not be stored in this repo. See effects how it does this. They need to be in a folder system that is expected from the consumer, probably relative to a consumer input path that can be established on boot.

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`

### question-55
> Should shaders be able to write `flat slot assignment`, `material slot assignment`, `blur`, and `warble` all from one shader contract?

### truth-55
> 27 yes

- timestamp: `26 07 12 12 18 54`
- discord-message-id: `unavailable-in-operator-log`
