# camera presentation truth

### 2026-09-20-authored-frame-interpolation-not-free-orbit
- j-quote: "I just ran it, it's very flashy and I actually think it's disorienting and I don't think it looks good. I think that this old system that we had where we could interpolate between angles smoothly works retty well. Is this using the same perspective math that we have to do death and such? This is kind of what I'm thinking i'm thinking that we do something in between we do 6 facing angles with the rolls and we do interpolations of them like we had in the old system."
- interpretation: camera presentation interpolates one discrete six-swing/four-roll step at a time; it is not free orbiting or a second depth-perspective system.

### 2026-09-20-roll-residual-handedness
- j-quote: "ok better. rolls do the smooth animate the wrong direction, lets look at that. left roll should do what the right roll does and vise versea -- for just the smooth part of the tran sition. the hard transition works."
- interpretation: preserve semantic roll commands and their hard midpoint handoff; reverse only the render-only roll residual's screen-space direction.

### 2026-09-20-swing-residual-handedness
- j-quote: "that looks tons better. lets swap the soft transitions of the swings as well, those may be reversed"
- interpretation: preserve authored swing semantics and hard midpoint commits; reverse only their render-only smooth residual direction.
