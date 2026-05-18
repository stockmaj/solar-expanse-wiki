# Achievements

Steam achievements available in **Solar Expanse**, with how to earn each.
Sourced from the game's `ContractDefinition` and `SpacecraftType` tables —
every binding here corresponds to an in-game trigger that awards the
achievement.  Each contract / spacecraft name links to its row on the
relevant page.

## By contract

| Achievement | How to earn (contract) | Condition |
| --- | --- | --- |
| Not Today | [Avoiding Armageddon (contract)](../contracts/#contract-contract-asteroid-impact) | — |
| Asteroid Colony | [Asteroid Colony (contract)](../contracts/#contract-contract-asteroid-outpost) | — |
| Fancy Way of Throwing Rocks | [Asteroid Pulling (contract)](../contracts/#contract-contract-asteroid-pulling) | — |
| To Infinity | [Beyond the Solar System (contract)](../contracts/#contract-contract-general-interstellar2) | — |
| Wanderlust | [Beyond the Solar System (contract)](../contracts/#contract-contract-general-interstellar2) | By year 2400 |
| Mars Colony | [Mars Colony (contract)](../contracts/#contract-contract-mars-colony1) | — |
| Humans on Mars | [Humans on Mars (contract)](../contracts/#contract-contract-mars-marslanding) | — |
| Mars Terraformed | [Mars Atmosphere (contract)](../contracts/#contract-contract-mars-terraform-atmo2) | After [Blue Mars](../contracts/#contract-contract-mars-terraform-water) |
| Terraform | [Mars Atmosphere (contract)](../contracts/#contract-contract-mars-terraform-atmo2) | After [Blue Mars](../contracts/#contract-contract-mars-terraform-water)<br>By year 2600 |
| Begin Terraforming | [Artificial Magnetosphere for Mars (contract)](../contracts/#contract-contract-mars-terraform-magnet) | — |
| Mars Terraformed | [Blue Mars (contract)](../contracts/#contract-contract-mars-terraform-water) | After [Mars Atmosphere](../contracts/#contract-contract-mars-terraform-atmo2) |
| Terraform | [Blue Mars (contract)](../contracts/#contract-contract-mars-terraform-water) | After [Mars Atmosphere](../contracts/#contract-contract-mars-terraform-atmo2)<br>By year 2600 |
| Moonbase Alpha | [Moonbase Alpha (contract)](../contracts/#contract-contract-moon-moonbase) | — |
| Titan Landing | [Titan Landing (contract)](../contracts/#contract-contract-outer-titanlanding) | — |
| First Orbit | [First Orbit (contract)](../contracts/#contract-contract-tutorial-firstorbit) | — |
| Lunar Landing | [Lunar Landing (contract)](../contracts/#contract-contract-tutorial-moonlanding) | — |
| Gravitational Slingshot | [Gravitational Slingshot (contract)](../contracts/#contract-contract-tutorial-slingshot) | — |

## By spacecraft

| Achievement | Trigger spacecraft |
| --- | --- |
| The Power of a Star | [Atlas (spacecraft)](../spacecraft/#spacecraft-spacecraft-asteroid-puller) |
| Old Reliable | [Stratos (spacecraft)](../spacecraft/#spacecraft-spacecraft-chem-large) |
| Do Astronauts Dream of Electric Ship | [Selene (spacecraft)](../spacecraft/#spacecraft-spacecraft-chem-mid2) |
| Old Reliable | [Iris (spacecraft)](../spacecraft/#spacecraft-spacecraft-chem-small) |
| Do Astronauts Dream of Electric Ship | [Athena (spacecraft)](../spacecraft/#spacecraft-spacecraft-electric-mid) |
| Do Astronauts Dream of Electric Ship | [Hermes (spacecraft)](../spacecraft/#spacecraft-spacecraft-electric-small) |
| The Power of a Star | [Zeus (spacecraft)](../spacecraft/#spacecraft-spacecraft-fusion-large) |
| The Power of a Star | [Sirius (spacecraft)](../spacecraft/#spacecraft-spacecraft-fusion-mid) |
| The Power of a Star | [Nike (spacecraft)](../spacecraft/#spacecraft-spacecraft-fusion-small) |
| Uranium Fever | [Ariane (spacecraft)](../spacecraft/#spacecraft-spacecraft-nuke-large) |
| Uranium Fever | [Hephaistos (spacecraft)](../spacecraft/#spacecraft-spacecraft-nuke-mid) |
| Uranium Fever | [Cronos (spacecraft)](../spacecraft/#spacecraft-spacecraft-nuke-nolv) |
| Uranium Fever | [Prometheus (spacecraft)](../spacecraft/#spacecraft-spacecraft-nuke-small) |
| On Winds of Sunshine | [Zephyr (spacecraft)](../spacecraft/#spacecraft-spacecraft-sail-long) |
| On Winds of Sunshine | [Talos (spacecraft)](../spacecraft/#spacecraft-spacecraft-sail-mid) |
| On Winds of Sunshine | [Daedalus (spacecraft)](../spacecraft/#spacecraft-spacecraft-sail-small) |

## Notes

- Some contracts bind more than one achievement (e.g. *Interstellar 2* awards
  both *To Infinity* and *Wanderlust* — the latter only if completed before
  the year 2400).  Each binding appears as its own row.
- Spacecraft-bound achievements typically fire the first time you operate a
  craft of that propulsion class — building, fueling, or launching one
  depending on the achievement.
- The Condition column lists the extra requirements parsed from each
  binding's `conditions[]` array — typically a year deadline or a
  prerequisite contract.  "—" means the achievement fires the moment the
  parent contract is completed, with no further constraint.
- Achievement names are derived from the in-game id when no localized
  display name is available; the in-game UI may polish the wording further.

## See also

- [Contracts](../contracts/) — full contract list and dependency chain
- [Spacecraft](../spacecraft/)
- [Launch Vehicles](../launch-vehicles/)
