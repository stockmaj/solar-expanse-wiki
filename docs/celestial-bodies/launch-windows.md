# Launch Windows

The game doesn't store launch-window dates as static data — the porkchop
plot you see in Plan Mission is computed live from current planetary
positions. What *is* knowable in advance is the **synodic period**: how
often a given pair of bodies returns to the same relative geometry. After
each synodic period, the same Hohmann-style launch opportunity comes around
again.

Synodic periods below are computed from each body's semi-major axis using
Kepler's third law (`T_years = a^(3/2)`) and `synodic = 1 / |1/T_inner −
1/T_outer|`.

| Body | Semi-major axis (AU) | Orbital period | Earth ↔ body window |
| --- | --- | --- | --- |
| **Mercury** | 0.387 | 0.24 yr | 116 days (~3.8 months) |
| **Venus** | 0.723 | 0.62 yr | 584 days (~19.2 months) |
| **Mars** | 1.524 | 1.88 yr | 2.1 years |
| **Jupiter** | 5.203 | 11.87 yr | 399 days (~13.1 months) |
| **Saturn** | 9.537 | 29.45 yr | 378 days (~12.4 months) |
| **Uranus** | 19.189 | 84.06 yr | 370 days (~12.1 months) |
| **Neptune** | 30.070 | 164.89 yr | 367 days (~12.1 months) |
| **Pluto** | 39.482 | 248.09 yr | 367 days (~12.0 months) |
| **1 Ceres** | 2.768 | 4.61 yr | 467 days (~15.3 months) |
| **4 Vesta** | 2.362 | 3.63 yr | 504 days (~16.6 months) |
| **2 Pallas** | 2.770 | 4.61 yr | 466 days (~15.3 months) |

## Practical reading

- **Earth → Mercury** opens most often — ~116 days, less than every 4 months.
- **Earth → Venus** ~19 months.
- **Earth → Mars** opens roughly every 26 months — every mid-game player has
watched their cargo manifest waiting for one of these.
- **Earth → Jupiter and beyond** are short intervals (~13 months) because the
outer planets move slowly relative to Earth, so Earth laps them almost
yearly.  The Hohmann transfer itself takes years.
- **Earth → asteroid belt** (Ceres, Vesta, Pallas) sits between Mars and
Jupiter — windows ~14–16 months.

Moons aren't in this table — launching from Earth to the Moon (or Phobos,
Europa, etc.) doesn't have a useful synodic period; you just wait for your
spacecraft to be ready and the in-game flight planner handles phasing.

## See also

- [Planets](planets.md)
- [Celestial Bodies overview](README.md)
