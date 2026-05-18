# Solar Expanse Wiki

A player-facing reference for **[Solar Expanse](https://store.steampowered.com/app/1369700/)** —
the realistic solar-system management game by Maciej Miąsik / TJ Entertainment.

This wiki is built from the game's own localization files and asset bundles, so
the names, descriptions, and stat tables here match exactly what you see in-game.

## Contents

| Section | What's in it |
| --- | --- |
| **[Celestial Bodies](celestial-bodies/)** | The Sun, planets, moons, asteroids, comets, and exoplanet systems. |
| [Spacecraft](spacecraft/) | Interplanetary craft — Iris, Selene, Stratos, Hermes, Centaur, Athena, Prometheus, Hephaistos, Ariane, Cronos, Nike, Sirius, Zeus. |
| [Launch Vehicles](launch-vehicles/) | Surface-to-orbit lifters — Albatross, Pelican, Magpie, Condor, Teratorn. |
| [Facilities](facilities/) | Ground buildings and orbital modules — power, mining, refining, habitats, life support, etc. |
| [Research](research/) | Tech tree — chemical, electric, nuclear, fusion propulsion, life support, materials, computing. |
| [Missions](missions/) | Mission planning — Plan Mission walk-through, mission types, launch-window pointer. |
| [Contracts](contracts/) | Story and freelance contracts — the in-game Contracts tab — that drive progression. |
| [Resources](resources/) | The 20+ resource types — water, metals, fissiles, He-3, supplies, exotic alloys. |
| [Asteroid Taxonomy](asteroid-taxonomy/) | The five asteroid classes (Carbon, Dark, Helium-3, Metal, Stone) and the per-class resource roll table the game uses when you mine a deposit. |
| [Corporations](corporations/) | Playable starting factions — SoleX, NASA, ESA, CNSA, Roscosmos. |

## How to use this wiki

Every page is plain Markdown. Jekyll renders the site on GitHub Pages, with a
custom layout, a sortable-table snippet, and three small browser-side modules
that power the launch-window, gravity-assist, and corporation-comparison
calculators. Browse by clicking section links above.

## Contributing

Almost every page is generated from the game's own files; direct edits get
overwritten when the pipeline reruns. Fixes belong in the [generator code](https://github.com/stockmaj/solar-expanse-wiki/tree/main/extract).
See [CONTRIBUTING](CONTRIBUTING.md) for details.

## Credits

- **Solar Expanse** © Maciej Miąsik / TJ Entertainment.
- Wiki text is generated from the game's English localization and is presented
here for reference purposes only.
