# Celestial Bodies

All natural objects in Solar Expanse — from the nine planets, through
moons and asteroid belts, out to comets and the Trappist-1 and Kepler-90
exoplanet systems reachable in the late game.

| Type | Count | Notes |
| --- | --- | --- |
| **[Planets](planets.md)** | 9 | Major body orbiting the Sun. Most planets host one or more moons. |
| **[Moons](moons.md)** | 25 | Natural satellite orbiting a planet. |
| **[Asteroids](asteroids.md)** | 52 | Small body. Some are in the main belt, some are near-Earth, and some co-orbit Jupiter at the Trojan/Greek points. Asteroids can be pulled into new orbits with an Asteroid Engine Module. |
| **[Comets](comets.md)** | 8 | Periodic body on a highly eccentric orbit. |
| **[Exoplanets](exoplanets.md)** | 15 | Body in a non-Solar system. Reachable only via a generation ship. |

## Orbital data

Orbital elements below are anchored at the **2020-01-01 campaign-start epoch**
the game ships — the same epoch the in-game flight planner uses for its
initial body positions.

| Field | Meaning | Unit |
| --- | --- | --- |
| Mass | Body mass | 10²⁴ kg |
| Radius | Mean radius | km |
| Semi-major axis | Average orbital radius (around the Sun for planets, around the parent for moons) | AU (planets), km (moons) |
| Eccentricity | Orbital ellipticity (0 = circular) | dimensionless |
| Inclination | Tilt relative to the ecliptic | degrees |

## Habitability

The Object Info window grades every body on four habitability axes:

| Axis | Labels (worst → best) |
| --- | --- |
| Temperature | Extremely Cold · Cold · Temperate · Hot · Extremely Hot · Melting Hot |
| Atmosphere | No Atmosphere · Thin Atmosphere · Earth-like Atmosphere · Non-breathable · High Pressure · Extreme Pressure |
| Gravitation | Extreme Gravity · High Gravity · Standard Gravity · Low Gravity · Minimal Gravity · 0g |
| Radiation | No Radiation · Minor · Noticeable · Significant · Serious hazard · Extreme hazard |

Combined into a single **Habitability %**, with crew status:

| Habitability | Crew status |
| --- | --- |
| Excellent (≈100%) | A perfect place for life. |
| Good | Our crews can live here with minor issues. |
| Marginal | Our crews will struggle to survive here. |
| Hostile | Our crews cannot land here — the object is too hostile. |

Habitability can be improved through terraforming.

## Pages

- [Planets](planets.md) — the nine major bodies
- [Moons](moons.md) — natural satellites of each planet
- [Asteroids](asteroids.md) — main belt, NEOs, Trojans/Greeks, and others
- [Comets](comets.md) — periodic comets
- [Exoplanets](exoplanets.md) — Trappist-1 and Kepler-90 systems
- [Launch Windows](launch-windows.md) — synodic periods for planning Earth → body missions
- [Initial habitability per scenario](scenario-state.md) — start-of-scenario temperature, pressure, gravity, water, radiation, etc. for each named body, compared across the four pre-built saves
