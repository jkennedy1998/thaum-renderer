# thaum-renderer/domain/command-bar design truths

### 2026-10-11T18-22-45Z-defer-sticky-side-module-migration
- j-quote: "i think we can delay this edit to modules to make them sticky to sides. make a note in the command bar about that. once that gets better designed we can then do that migration / change."
- interpretation: the ScreenAnchor-style sticky-edge module migration is deliberately deferred; the command bar keeps its current direct screen anchoring until that design matures.

### 2026-10-11T18-22-45Z-command-bar-vertical-ordering
- j-quote: "the command bar as it is now, lets make the vertical ordering change a bit. _____ border____ (dont change) item on list scrollbar here space bottom of screen"
- interpretation: expanded bar stack top-to-bottom is border (unchanged), button row, scrollbar row, then blank space down to the screen edge; the scrollbar moved from just under the border to just under the button row.
