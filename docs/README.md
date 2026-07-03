# Solar Expanse Wiki

A player-facing reference for **[Solar Expanse](https://store.steampowered.com/app/1369700/)** —
the realistic solar-system management game by SpaceOps.

This wiki is built from the game's own localization files and asset bundles, so
the names, descriptions, and stat tables here match exactly what you see in-game.

## Contents

| Section | What's in it |
| --- | --- |
| **[Celestial Bodies](celestial-bodies/)** | The Sun, planets, moons, asteroids, comets, and exoplanet systems. |
| [Exoplanets](exoplanets/) | Destination systems reachable via a generation ship. |
| [Spacecraft](spacecraft/) | Interplanetary craft. |
| [Launch Vehicles](launch-vehicles/) | Surface-to-orbit lifters. |
| [Facilities](facilities/) | Ground buildings and orbital modules — power, mining, refining, habitats, life support, etc. |
| [Transportable Modules](transportable-modules/) | Spacecraft payload — mining rigs, refiners, probes, telescopes, habitats, power plants, and crew compartments. |
| [Research](research/) | Tech tree. |
| [Missions](missions/) | Mission planning — Plan Mission walk-through, mission types, launch-window pointer. |
| [Contracts](contracts/) | Story and freelance contracts — the in-game Contracts tab — that drive progression. |
| [Achievements](achievements/) | Steam achievements and how to earn each — keyed to contracts, spacecraft, and launch vehicles. |
| [Resources](resources/) | Resource catalogue — production, consumption, and per-body mining license fees. |
| [Asteroid Taxonomy](asteroid-taxonomy/) | Asteroid classes and the per-class resource roll table the game uses when you mine a deposit. |
| [Terraforming](terraforming/) | Per-resource thermal / phase constants — boiling and melting points, latent heat, heat capacity, optical depth — that drive the atmosphere sim. |
| [Corporations](corporations/) | Playable starting factions. |

## How to use this wiki

- **Find data fast.** Bodies (planets, moons, asteroids, comets, exoplanets) live under [Celestial Bodies](celestial-bodies/) — radius, semi-major axis, eccentricity, inclination, parent. Fleet planning lives under [Spacecraft](spacecraft/) and [Launch Vehicles](launch-vehicles/) — dry mass, cargo, fuel, thrust, exhaust velocity, build cost. What-to-build prompts and the workforce / energy / resource math behind each structure are on [Facilities](facilities/). The tech tree — costs, prereqs, and what each node unlocks — is on [Research](research/).
- **Plan progression.** [Contracts](contracts/) is the in-game contracts tab, ordered by their root tree, with rewards and follow-on links. [Missions](missions/) walks the Plan Mission flow and points at launch-window data. [Achievements](achievements/) lists every Steam achievement keyed to the contract, spacecraft, or launch vehicle that earns it.
- **Compare scenario starts.** [Corporations](corporations/) is a side-by-side table of the playable factions — starting cash, starting research, starting fleet, starting facilities — so you can pick the run you want.
- **Understand the economy.** [Resources](resources/) lists every resource, what produces it, what consumes it, and per-body mining license fees.  [Asteroid Taxonomy](asteroid-taxonomy/) shows the resource roll table for each asteroid class so you know what mining a given asteroid will yield.
- **Tables are sortable.** Click any column header to sort by that column; click again to reverse.  Hover a column header for a tooltip explaining its units or source data.
- **Calculator.** Several pages embed a small Calculator that computes a fleet's total payload and crew capacity for trip planning — change the inputs and the totals update live.

## Contributing

Almost every page is generated from the game's own files; direct edits get
overwritten when the pipeline reruns. Fixes belong in the [generator code](https://github.com/stockmaj/solar-expanse-wiki/tree/main/extract).
See [CONTRIBUTING](CONTRIBUTING.md) for details.

## Credits

- **Solar Expanse** © SpaceOps.
- Wiki text is generated from the game's English localization and is presented
here for reference purposes only.
