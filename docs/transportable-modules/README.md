# Transportable Modules

Spacecraft payload — mining rigs, refiners, probes, telescopes, habitats, power
plants, and crew compartments that ride on (or alongside) interplanetary craft.
Most are loaded as cargo on a launch vehicle and deployed at the destination;
a few are assembled directly in an orbital shipyard. Modules are grouped by
the gameplay role they fill on the destination — Mining rigs extract
resources, Probes survey a body, Crew Capacity modules house a population,
and so on.

## Mining

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day)">Role</span> | <span title="Which resources the rig can extract">Mines</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Metal Mining Module** | 5 | Mining (0.3) | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;Metals</span> | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;5</span> | 50 | 300 | Equipment for extraction of common construction metals such as iron, aluminum, and silicon. |
| **Volatile Refining Module** | 5 | Mining (0.3) | <span style="white-space:nowrap" title="Water"><img src="../images/resources/water.png" width="16" alt="Water"/>&nbsp;Water</span><br><span style="white-space:nowrap" title="Carbon"><img src="../images/resources/volatile.png" width="16" alt="Carbon"/>&nbsp;Carbon</span> | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;5</span> | 50 | 300 | Equipment for extracting and refining of volatile compounds such as water, nitrogen, and various carbon molecules. |
| **Mining Module** | 15 | Mining (0.2) | <span style="white-space:nowrap" title="Water"><img src="../images/resources/water.png" width="16" alt="Water"/>&nbsp;Water</span><br><span style="white-space:nowrap" title="Carbon"><img src="../images/resources/volatile.png" width="16" alt="Carbon"/>&nbsp;Carbon</span><br><span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;Metals</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;Rare Metals</span><br><span style="white-space:nowrap" title="Silicon"><img src="../images/resources/silicon.png" width="16" alt="Silicon"/>&nbsp;Silicon</span> | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;75</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;10</span> | 30 | 900 | A mobile set of equipment for extracting resources locally, when no infrastructure exists yet. |
| **Rare Metal Extraction Module** | 5 | Mining (0.2) | <span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;Rare Metals</span> | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;5</span> | 50 | 300 | Equipment for extracting rare metallic elements such as gold, tungsten, titanium, beryllium, and various radioactive isotopes. |
| **Mining Equipment** | — | Mining (0.1) | — | Yes | — | 7 | 210 |  |

## Refining

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Mobile Refinery** | 25 | Refining | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;120</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;20</span> | 60 | 210 | Processes Water into Fuel. |

## Probes

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Ground Probe** | 1 | Probe (10) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;50</span> | 100 | 300 | Searches for resources on an object where it is located. |
| **Probe** | 1 | Probe (10) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;5</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;1</span> | 35 | 300 | Searches for resources on an object where it is located. |
| **Probe** | — | Probe (1) | Yes | — | 7 | 210 |  |

## Crew Transport

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Crew Compartment Type-L** | 60 | Crew transport (100) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;20</span><br><span style="white-space:nowrap" title="Polymers"><img src="../images/resources/plastic.png" width="16" alt="Polymers"/>&nbsp;100</span> | 90 | 300 | <sub>Locked at start (unlocks: [STO Transports](../research/#research-research-lifesup-4))</sub><br>Module allowing for transportation of 100 humans |
| **Crew Compartment Type-M** | 15 | Crew transport (20) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;10</span><br><span style="white-space:nowrap" title="Polymers"><img src="../images/resources/plastic.png" width="16" alt="Polymers"/>&nbsp;20</span> | 30 | 300 | <sub>Locked at start (unlocks: [Improved Capsule Construction](../research/#research-research-lifesup-2))</sub><br>Module allowing for transportation of 20 humans |
| **Crew Compartment Type-S** | 5 | Crew transport (5) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;5</span><br><span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;20</span> | 10 | 300 | Module allowing for transportation of 5 humans |

## Crew Capacity

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Mobile Habitat** | 80 | Crew capacity (20) | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;120</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;40</span><br><span style="white-space:nowrap" title="Supplies"><img src="../images/resources/supply.png" width="16" alt="Supplies"/>&nbsp;40</span> | 120 | 300 | Long-term shelter with integrated power source, can be packed as cargo and deployed immediately to house a crew of 20 without additional infrastructure on a location. Crew within still requires supplies as in any other habitat. |

## Power Generation

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Radioisotope Generator** | 5 | Energy production + Energy storage (2) | Yes | <span style="white-space:nowrap" title="Exotic Alloys"><img src="../images/resources/alloy.png" width="16" alt="Exotic Alloys"/>&nbsp;70</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;35</span><br><span style="white-space:nowrap" title="Fissiles"><img src="../images/resources/uran.png" width="16" alt="Fissiles"/>&nbsp;10</span> | 120 | 30 | Produces constant power from radioactive decay, without requiring input or maintenance |

## Construction

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Construction Equipment** | 5 | Construction | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;5</span> | 30 | 300 | Allows construction of facilities. |

## Installation

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Space Telescope** | 2 | Installation | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;10</span><br><span style="white-space:nowrap" title="Glass"><img src="../images/resources/glass.png" width="16" alt="Glass"/>&nbsp;30</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;5</span> | 30 | 300 | Searches distant objects for resources. |
| **Orbital Construction** | 50 | Installation | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;100</span><br><span style="white-space:nowrap" title="Electronics"><img src="../images/resources/chips.png" width="16" alt="Electronics"/>&nbsp;10</span> | 100 | 300 | Allows construction of orbital facilities. Cannot be transported once placed in orbit. |

## Ship Construction

| Module | <span title="Dry mass in tonnes">Mass (t)</span> | <span title="Module role and its magnitude (mining rate per day, crew capacity, energy production, …)">Role</span> | <span title="Whether the module can be loaded into a launch vehicle as cargo">Cargo</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Monthly maintenance cost ($/30-day month) — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.">Maint ($/mo)</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Hermes Part** | 100 | Ship construction | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;100</span> | 7 | 210 |  |

## Reading the table

- **Mass** is the module's dry mass in tonnes — included in the carrier launch vehicle's payload budget.
- **Role** is what the module does on station — mining a resource at the listed rate, refining one resource into another, transporting a fixed crew count, etc. The number in parentheses is the role-specific magnitude (mining rate per day, crew capacity, energy production).
- **Mines** (Mining section only) lists the resources a rig can extract from the body it lands on.
- **Cargo** indicates whether the module can ride to orbit inside a launch vehicle's cargo bay. Modules that say *No* must be assembled directly in an orbital shipyard.
- **Build cost / Time** are the resources and days required to build the module — either on the surface (most cargo-loadable modules) or in orbit.
- **Maint ($/mo)** is the monthly cash upkeep while the module is active — the dump stores a per-day rate; we multiply by 30 to match the in-game UI.
- **Locked at start** rows are unavailable until you research the listed technology (or, when no research direct-unlocks the module, until the scenario or contract chain grants it).

## See also

- [Spacecraft](../spacecraft/) — the craft these modules ride on
- [Launch Vehicles](../launch-vehicles/) — surface-to-orbit lifters
- [Facilities](../facilities/) — ground and surface-installed structures
- [Research](../research/) — technology unlocks for locked modules
