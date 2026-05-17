# Launch Vehicles

Surface-to-orbit lifters. Every spacecraft that's built on a planet's surface
has to ride one of these to reach orbit, and the launch cost paid here is paid
on **every** launch — reusable vehicles amortise their build cost over many
flights.

Three propulsion families are unlocked across the tech tree:

- **Chemical** rockets — kerosene/RP-1 burned with LOX. The early- and mid-game default.
- **Nuclear-thermal** rockets — hydrogen heated by a fission reactor and expelled as reaction mass. Higher specific impulse for the same payload class; unlocked later in the tech tree.
- **Mechanical / magnetic** launchers — non-rocket systems built as facilities. See [Alternative launch methods](#alternative-launch-methods) below.

## Chemical rockets

| Launch Vehicle | <span title="Max payload to low orbit, in tonnes">Payload (t)</span> | <span title="Survives reentry and can fly again (Yes / Partial / No)">Reusable</span> | <span title="Crew-rated for human passengers">Crew</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Cash fee paid on every launch">Launch</span> | <span title="Daily maintenance cost while idle on the pad">Maint</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| <a id="lv-id-Rocket-RocketType1"></a>**Sparrow** | 10 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;50</span> | 50 | 150 | 10 | Cheap, simple, single use small Launch Vehicle. |
| <a id="lv-id-Rocket-RocketType5"></a>**Al-Ice Rocket** | 40 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;20</span><br><span style="white-space:nowrap" title="Water"><img src="../images/resources/water.png" width="16" alt="Water"/>&nbsp;20</span> | 25 | 0 | 0 | Aluminium-Ice LV. Low efficiency but easily assembled from local resources. |
| <a id="lv-id-Rocket-RocketType3"></a>**Falcon** | 42 | Partial | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;80</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;15</span> | 90 | 400 | 20 | Reusable Medium Launch Vehicle |
| <a id="lv-id-Rocket-RocketType2"></a>**Kestrel** | 64 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;100</span> | 70 | 300 | 10 | Medium Launch Vehicle. |
| <a id="lv-id-Rocket-RocketType6"></a>**Al-Ice Rocket (Heavy)** | 100 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Water"><img src="../images/resources/water.png" width="16" alt="Water"/>&nbsp;50</span> | 50 | 0 | 0 | Aluminium-Ice LV. Low efficiency but easily assembled from local resources. Larger variant for heavy payloads. |
| <a id="lv-id-Rocket-RocketType7"></a>**Hawk** | 200 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;120</span> | 120 | 500 | 10 | Old but reliable super heavy launch vehicle. |
| <a id="lv-id-Rocket-RocketType4"></a>**Eagle** | 800 | Partial | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;250</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;25</span> | 110 | 3k | 20 | Reusable Super Heavy-class Launch Vehicle |
| <a id="lv-lv-chem-seadragon"></a>**Albatross** | 1800 | No | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;800</span> | 180 | 4.6k | 20 |  |
| <a id="lv-lv-chemadvanced"></a>**Condor** | 8000 | Partial | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;500</span><br><span style="white-space:nowrap" title="Polymers"><img src="../images/resources/plastic.png" width="16" alt="Polymers"/>&nbsp;500</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;100</span> | 160 | 8k | 10 | Advanced chemical launch vehicle powered by polynitrogen fuel. |

## Nuclear-thermal rockets

| Launch Vehicle | <span title="Max payload to low orbit, in tonnes">Payload (t)</span> | <span title="Survives reentry and can fly again (Yes / Partial / No)">Reusable</span> | <span title="Crew-rated for human passengers">Crew</span> | <span title="Resources required to construct">Build cost</span> | <span title="Build time in days">Time (d)</span> | <span title="Cash fee paid on every launch">Launch</span> | <span title="Daily maintenance cost while idle on the pad">Maint</span> | Description |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| <a id="lv-lv-nuke-small"></a>**Pelican** | 1200 | Partial | Yes | <span style="white-space:nowrap" title="Metals"><img src="../images/resources/metal.png" width="16" alt="Metals"/>&nbsp;400</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;50</span><br><span style="white-space:nowrap" title="Fissiles"><img src="../images/resources/uran.png" width="16" alt="Fissiles"/>&nbsp;40</span> | 140 | 3k | 10 | Nuclear-boosted Single-Stage-To-Orbit (SSTO) launch vehicle. |
| <a id="lv-lv-nuke-mid"></a>**Magpie** | 4000 | Partial | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;500</span><br><span style="white-space:nowrap" title="Rare Metals"><img src="../images/resources/raremetal.png" width="16" alt="Rare Metals"/>&nbsp;100</span><br><span style="white-space:nowrap" title="Fissiles"><img src="../images/resources/uran.png" width="16" alt="Fissiles"/>&nbsp;100</span> | 180 | 4.8k | 10 | Powerful launch vehicle using gas core nuclear engines. |
| <a id="lv-lv-nuke-large"></a>**Teratorn** | 20000 | Partial | Yes | <span style="white-space:nowrap" title="Alloy"><img src="../images/resources/steel.png" width="16" alt="Alloy"/>&nbsp;1k</span><br><span style="white-space:nowrap" title="Exotic Alloys"><img src="../images/resources/alloy.png" width="16" alt="Exotic Alloys"/>&nbsp;100</span><br><span style="white-space:nowrap" title="Fissiles"><img src="../images/resources/uran.png" width="16" alt="Fissiles"/>&nbsp;200</span> | 360 | 15k | 10 | The largest and most powerful launch vehicle devised, the Teratorn can lift thousands of tons and quickly come back to lift another payload. |

## Reading the tables

- **Max Payload** is the heaviest load (in tonnes) the vehicle can carry to low orbit.
- **Reusable** — *Yes* means the vehicle survives reentry and can fly again; *No* means each launch consumes the vehicle.
- **Crew Rated** — whether the vehicle can carry humans, not just cargo.
- **Launch cost** is the cash fee paid every launch; **Maintenance** is the daily upkeep cost while idle on the pad.

## Alternative launch methods

The game also models several non-rocket launch systems unlocked through
research and built as facilities at the launch site:

| Method | Notes |
| --- | --- |
| **Launch Pad** | Organized launch infrastructure, reduces launch cost. |
| **MagRails** | Long ramp built atop suitable terrain, outfitted with MagLev tracks. |
| **Mass Driver** | Set of superconducting electromagnetic accelerators able to launch payloads directly into orbit. |
| **Magnetic Catapult** | Larger mass driver capable of launching payloads on interplanetary trajectories by itself. |
| **Spin Launcher** | Launches payloads via extremely high rotary acceleration. |
| **Space Elevator** | Supermaterial cable from surface to geostationary orbit. |

## See also

- [Spacecraft](../spacecraft/)
- [Research](../research/) — Launch Vehicles tech category
