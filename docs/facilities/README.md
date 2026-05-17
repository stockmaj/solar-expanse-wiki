# Facilities

Facilities are the buildings and modules you place on planets, moons, asteroids,
and in orbit. Each consumes power and workers, may require a research
prerequisite, and either produces, processes, or enables something — power
plants generate energy, refineries turn ore into refined metal, mines extract
raw resources, etc.

Facilities are split into two families:

- **Ground facilities** sit on a body's surface. They use local workers and
may need atmospheric conditions to function.
- **Orbital modules** attach to a space station or shipyard in orbit. They
don't need a habitable surface, but you have to build the station first.

## Ground facilities

| Facility | Type | Build cost | Workers | Energy/day | Maintenance | Research prereq | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Habitat** | Habitation | 100 Metals<br>20 Supplies | — | 1 | — | — | A system of structures for housing colonists. |
| **City** | Habitation | 500 Alloy<br>500 Glass<br>100 Supplies | — | 200 | — | — | A sprawling network of habitats and domes, able to house large numbers of colonists, with all the needed amenities. |
| **Dome Habitat** | Habitation | 200 Alloy<br>100 Glass<br>40 Supplies | — | 10 | — | — | Domed colonies, with significant free space and pressurized volume for comfortable habitation of communities. |
| **Underground Habitat** | Habitation | 20 Metals<br>20 Supplies | — | 1 | — | — | Underground structures, built in lava tubes and dug out cavities, are naturally sheltered from radiation and easier to construct, if limited in size. |
| **Outpost** | Habitation | 40 Metals<br>10 Supplies | — | — | 10 | — | A self-sufficient outpost for remote places. Allows a crew of 10 to operate in rough conditions without support, but is impractical to expand into a colony. |
| **City Station** | Habitation | 1k Metals<br>200 Rare Metals<br>100 Supplies<br>400 Glass | — | — | — | — | Vast, spinning ring with room for thousands of people to live. |
| **Rotating Habitat** | Habitation | 200 Metals<br>50 Rare Metals<br>20 Supplies | — | — | — | — | In absence of gravity, the habitat spins to compensate with centrifugal force. While providing a comfortable illusion of 1g, it requires much larger and complicated construction. |
| **Rotating Station** | Habitation | 400 Metals<br>100 Rare Metals<br>200 Glass<br>40 Supplies | — | — | — | — | A rotating wheel style station with room for habitation, work, and all aspects of deep space life. |
| **Space Elevator** | LaunchFacility | 100k Polymers<br>10k Exotic Alloys<br>40k Alloy | 50 | 4 | 100 | — | A supermaterial cable stretching from the surface to geostationary orbit. Allows reaching space just by riding an elevator up the cable. Uses an asteroid as a counterweight; once anchored, all resources from the asteroid will become available. |
| **Electromagnetic Catapult** | LaunchFacility | 500 Alloy<br>100 Electronics<br>300 Rare Metals | 20 | 2.5 | 50 | — | Larger and more powerful mass driver capable of launching payloads on interplanetary trajectories by itself. |
| **Magnetic Launch Rails** | LaunchFacility | 300 Alloy<br>200 Electronics<br>100 Rare Metals | 20 | 2.5 | 50 | — | Long ramp built atop suitable terrain, outfitted with MagLev tracks to accelerate payloads into space. |
| **Stationary Mass Driver** | LaunchFacility | 300 Alloy<br>40 Electronics<br>50 Rare Metals | — | 2.5 | 50 | — | Set of superconducting electromagnetic accelerators, able to launch payloads directly into orbit. |
| **Launch Pad** | LaunchFacility | 120 Metals | — | — | 20 | — | Organized launch infrastructure, reduces launch vehicle launch cost by 10%. |
| **Rotary Launcher** | LaunchFacility | 300 Alloy<br>20 Electronics | 20 | 1.5 | 50 | — | Launches payloads by utilizing extremely high rotary acceleration. |
| **Exotic Alloy Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Exotic Alloys from any kind of deposit on the object |
| **Carbon Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Carbon Mine** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts carbon from minerals and volatile carbon compounds. |
| **Carbon Mine** | Mining | 1.2k Metals | 50 | 5 | 10 | — | Extracts carbon from minerals and volatile carbon compounds. |
| **CO2 Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Carbon Dioxide Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts CO2 from any kind of deposit on the object |
| **Electronics Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Electronics from any kind of deposit on the object |
| **Fuel Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Fuel from any kind of deposit on the object |
| **Fuel Mine** | Mining | 1.2k Metals | 100 | 5 | 10 | — | Extracts fuel from organic material deposits. |
| **Glass Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Glass from any kind of deposit on the object |
| **Helium-3 Extractor** | Mining | 125 Metals | 20 | 1.2 | 10 | — | Processes enormous amounts of regolith to extract precious Helium-3 for use in fusion. |
| **HEL 3 Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Hydrogen Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Hydrogen Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Hydrogen from any kind of deposit on the object |
| **Water Ice Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts ice and hydrated minerals and refines them into water. |
| **Water Ice Extractor** | Mining | 1.2k Metals | 50 | 5 | 10 | — | Extracts ice and hydrated minerals and refines them into water. |
| **Metal Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Metal Mining Base** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts common metal elements and smelts them for use. |
| **Metal Mining Base** | Mining | 125 Metals | 50 | 5 | 10 | — | Extracts common metal elements and smelts them for use. |
| **Nitrogen Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Nitrogen Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Nitrogen from any kind of deposit on the object |
| **Noble Gas Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Noble Gas Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts non-reactive gases such as helium, xenon or argon. |
| **Noble Gas Extractor** | Mining | 125 Metals | 50 | 5 | 10 | — | Extracts non-reactive gases such as helium, xenon or argon. |
| **Oxygen Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Oxygen Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Oxygen from any kind of deposit on the object |
| **Polymers Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Polymers from any kind of deposit on the object |
| **Rare Metal Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Rare Metal Extractors** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts rare metallic elements such as tungsten, gold, or iridium. |
| **Rare Metal Extractors** | Mining | 1.2k Metals | 50 | 5 | 10 | — | Extracts rare metallic elements such as tungsten, gold, or iridium. |
| **Silicon Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Silicon Mine** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts silicon from regolith. |
| **Silicon Mine** | Mining | 1.2k Metals | 50 | 5 | 10 | — | Extracts silicon from regolith. |
| **Orbital Gas Extractor** | Mining | 150 Alloy<br>20 Polymers<br>10 Electronics | — | — | 1 | — | Extracts gases from planetary atmosphere. Can also extract gases from gas giants. |
| **Steel Extractor** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Extracts Steel from any kind of deposit on the object |
| **Uran Release Station** | Mining | 125 Metals | 5 | 0.5 | 10 | — | Releases {0} to the surface |
| **Fissiles Mine** | Mining | 125 Metals | 10 | 1 | 10 | — | Extracts fissile elements such as uranium and thorium. |
| **Fissiles Mine** | Mining | 125 Metals | 50 | 5 | 10 | — | Extracts fissile elements such as uranium and thorium. |
| **Helium-3 Factory** | Other | 200 Metals<br>20 Electronics<br>20 Fissiles | — | — | 100 | — | Produces He-3 through creation and decay of radioactive isotopes. |
| **Mining Facility** | Other | 100 Metals<br>10 Electronics | — | — | 100 | — | An immobile mining complex capable of extracting large quantities of resources. |
| **Fissile Extraction Facility** | Other | 200 Metals<br>20 Electronics | — | — | 100 | — | Extraction and refinement of radioactive materials. |
| **Headquarters** | Other | 1k Metals | — | — | 100 | — | Center of operations. Heart of the company. |
| **Research Laboratory** | Other | 100 Alloy<br>10 Rare Metals<br>10 Electronics<br>10 Glass | 20 | 2 | 10 | — | Boosts research speed by 3%. |
| **Observatory** | Other | 40 Alloy<br>50 Glass<br>10 Electronics | 5 | 1 | 10 | — | Searches distant objects for resources. |
| **Telescope** | Other | 10 Alloy<br>30 Glass<br>5 Electronics | — | — | 10 | — | Searches distant objects for resources. |
| **Batteries** | Power | 40 Rare Metals<br>30 Alloy<br>10 Electronics | — | — | 10 | — | Battery banks storing power for future use. |
| **Carbon Power Plant** | Power | 730 Alloy<br>70 Exotic Alloys | 200 | — | 10 | — | Produces power by burning Carbon |
| **Chemical Reactor** | Power | 400 Alloy<br>300 Exotic Alloys<br>50 Electronics | 100 | — | 25 | — | Produces power from exothermic reactions of locally extracted compounds. |
| **Power Plant** | Power | 200k Alloy | — | — | 10 | — | Produces power from burning fuel. |
| **Fusion Reactor** | Power | 300 Alloy<br>550 Exotic Alloys<br>300 Electronics | 200 | — | 350 | — | Produces power from nuclear fusion. |
| **Geothermal Power** | Power | 400 Alloy<br>250 Exotic Alloys<br>100 Electronics | 100 | — | 25 | — | Produces power from energy extracted underground. |
| **Geothermal Power** | Power | 200k Alloy | — | — | 10 | — | Produces power from energy extracted underground. |
| **Hydrogen Power Plant** | Power | 500 Alloy<br>50 Exotic Alloys<br>70 Electronics | 150 | — | 50 | — | Produces power from burning Hydrogen |
| **Nuclear Reactor** | Power | 1.3k Alloy<br>200 Exotic Alloys<br>100 Electronics | 200 | — | 20 | — | Produces power from a controlled chain reaction of fissile elements. |
| **Nuclear Reactor** | Power | 100k Alloy<br>50k Rare Metals<br>50k Electronics | — | — | 10 | — | Produces power from a controlled chain reaction of fissile elements. |
| **Remote Power Receiver** | Power | 130 Glass<br>50 Alloy<br>20 Rare Metals | — | — | 95 | — | A large complex of rectennas and panels that receive remotely transferred power from orbits or other objects. |
| **Orbital Power Receiver** | Power | 130 Glass<br>50 Alloy<br>20 Rare Metals | — | — | 95 | — | Receives power in orbit. |
| **Solar Array** | Power | 50 Glass<br>10 Alloy | 2 | — | 3 | — | Produces power from solar radiation. Power production depends on distance from the Sun |
| **Solar Array** | Power | 50k Metals<br>20k Silicon | — | — | 10 | — | Produces power from solar radiation. Power production depends on distance from the Sun. |
| **Remote Power Emitter** | Power | 100 Glass<br>20 Electronics<br>80 Exotic Alloys<br>150 Alloy | — | 1000 | 700 | — | Transfers power from surface to orbit. |
| **Orbital Power Station** | Power | 120 Glass<br>30 Alloy | — | — | 10 | — | Generates power in orbit and transfers it to receivers. |
| **Wind Power** | Power | 80 Alloy | 2 | — | 3 | — | Produces power from atmospheric movements. |
| **Wind Power** | Power | 60k Alloy | — | — | 10 | — | Produces power from atmospheric movements. |
| **Alloy Smelting** | Production | 300 Metals | 10 | 2.5 | 10 | — | Produces Alloy from Metal. |
| **Alloy Smelting** | Production | 300k Metals | 100 | 15 | 10 | — | Produces Alloy from Metal. |
| **Co2 Electrolysis** | Production | 50 Metals | 10 | 2.5 | 10 | — | Splits CO2 into carbon and oxygen |
| **Consumer Goods Factory** | Production | 200k Alloy<br>10k Electronics<br>40k Polymers | — | 1.5 | 10 | — | Produces consumer goods from manufactured resources |
| **Agriculture Complex** | Production | 200 Alloy<br>50 Electronics<br>100 Glass<br>200 Polymers | 100 | 20 | 50 | — | System of farms and fields, growing crops in local soil. |
| **Electrolysis Plant** | Production | 50 Metals | 10 | 2.5 | 10 | — | Produces Hydrogen and Oxygen from Water using electrolysis. |
| **Electrolysis Plant** | Production | 50k Metals | 100 | 25 | 10 | — | Produces Hydrogen and Oxygen from Water using electrolysis. |
| **Electronics Factory** | Production | 200 Alloy<br>10 Electronics | 40 | 2 | 10 | — | Produces Electronics. |
| **Electronics Factory** | Production | 200k Alloy<br>10k Electronics | 100 | 15 | 10 | — | Produces Electronics. |
| **Exotic Alloy Production** | Production | 300 Alloy<br>20 Electronics<br>50 Rare Metals | 20 | 1.5 | 10 | — | Utilizes Rare Metals and Fissiles to produce Exotic Alloys. |
| **Exotic Alloy Production** | Production | 300k Alloy<br>20k Electronics<br>50k Rare Metals | 100 | 15 | 10 | — | Utilizes Rare Metals and Fissiles to produce Exotic Alloys. |
| **Hydroponic Farm** | Production | 50 Metals | 5 | 0.5 | 10 | Space Farming | A closed, controlled ecosystem for growing plants and recycling atmosphere. |
| **Fuel Refinery** | Production | 50 Metals<br>10 Rare Metals | 5 | 1 | 10 | — | Turns Water into Chemical Fuel. |
| **Fuel Refinery** | Production | 50k Metals<br>10k Rare Metals | 50 | 10 | 10 | — | Turns Hydrogen and Oxygen into Chemical Fuel. |
| **Glass Kiln** | Production | 300 Alloy | 10 | 2 | 10 | — | Produces Glass from Silicon. |
| **Glass Kiln** | Production | 300k Alloy | 100 | 20 | 10 | — | Produces Glass from Silicon. |
| **Polymers Production** | Production | 200 Alloy | 20 | 1.5 | 10 | — | Produces Polymers from Carbon. |
| **Polymers Factory** | Production | 200 Alloy | 10 | 15 | 10 | — | Produces Polymers from Carbon. |
| **Orbital Construction** | Production | 100 Alloy<br>10 Electronics | — | — | 10 | — | Allows construction of orbital facilities. |
| **Orbital Shipyard** | Production | 150 Alloy<br>50 Polymers<br>25 Electronics | — | — | 10 | — | Allows construction of advanced spacecraft in orbit. |
| **Orbital Fuel Refinery** | Production | 10 Metals<br>30 Rare Metals | — | — | 10 | — | Turns Water into Chemical Fuel. |
| **Interstellar Vehicle Assembly** | Production | 200 Alloy | — | — | — | — | Construction site of a generation ship to travel between the stars. |
| **Vehicle Assembly** | Production | 200 Alloy<br>50 Rare Metals | 10 | 0.5 | 10 | — | Allows construction of spacecraft and launch vehicles on the surface. |
| **Planetary Magnetosphere Generator** | Terraformation | 36k Metals | — | 240 | 50 | — | Decreases radiation on the object's surface. |
| **Orbital Magnetosphere Generator** | Terraformation | 30k Alloy | — | 228 | 40 | — | Decreases radiation on the object's surface. |
| **Orbital Mirror** | Terraformation | 2k Polymers<br>27k Glass<br>1k Alloy | — | — | 10 | — | Increases the temperature on the object it's aimed at |
| **Orbital Shade** | Terraformation | 13k Alloy<br>6k Polymers<br>1k Exotic Alloys | — | — | 10 | — | Decreases the temperature on the object it's orbiting |

## Orbital modules

| Facility | Type | Build cost | Workers | Energy/day | Maintenance | Research prereq | Description |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Asteroid Engine** | Module | 5k Alloy<br>1k Polymers<br>1.1k Helium-3<br>1.5k Exotic Alloys | — | — | 10 | — | A mobile fusion-powered mass driver, ejecting regolith at enormous speed to push an asteroid into a desired orbit. Integrated high efficiency reactor does not require refueling. |

## Reading the table

- **Type** is the gameplay category — *Production*, *Mining*, *Storage*, *Power*, *Habitat*, etc. The Solar Expanse UI groups facilities by type when you open the build menu.
- **Workers** is the on-site population the facility needs to operate at full output. Most facilities throttle when understaffed.
- **Energy/day** is the running energy demand. Power facilities show this as `—`; everything else is a consumer.
- **Maintenance** is the per-day cash upkeep while the facility is active.
- **Research prereq** is the research that unlocks construction; `—` means it's available from the start (or the prereq lives outside the standard `lockByHelpNotUse` field, which a few specialist facilities use).

What this page does *not* show: per-facility produces / consumes rates and special-effect bonuses. Those are stored on dynamically-typed subclasses of each facility and aren't in the static descriptor data — the in-game tooltip is the source of truth for now.
